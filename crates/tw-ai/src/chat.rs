//! Document chat + lightweight RAG (F28.S3).
//!
//! Chunks the document by paragraph, retrieves relevant chunks via keyword
//! overlap (no embedding network in CI), and asks the AI with citations to
//! paragraph ids.

use tw_model::{Block, Document, NodeId};

use crate::context::DocumentContext;
use crate::provider::{AiError, CompletionRequest};
use crate::router::ProviderRouter;
use crate::task::AiTask;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

/// Reference back into the document (paragraph id for navigation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentReference {
    pub paragraph_id: NodeId,
    pub excerpt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    pub references: Vec<DocumentReference>,
}

/// One RAG chunk — typically one paragraph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentChunk {
    pub paragraph_id: NodeId,
    pub text: String,
    pub section_index: usize,
    pub block_index: usize,
}

impl DocumentChunk {
    pub fn token_estimate(&self) -> u32 {
        // Rough heuristic: ~4 chars per token.
        (self.text.chars().count() as u32 / 4).max(1)
    }
}

/// Split body paragraphs into RAG chunks (skips empty paragraphs).
pub fn chunk_document(doc: &Document) -> Vec<DocumentChunk> {
    let mut chunks = Vec::new();
    for (si, section) in doc.sections.iter().enumerate() {
        for (bi, block) in section.blocks.iter().enumerate() {
            let Block::Paragraph(para) = block else {
                continue;
            };
            let text = para.full_text();
            if text.trim().is_empty() {
                continue;
            }
            chunks.push(DocumentChunk {
                paragraph_id: para.id,
                text,
                section_index: si,
                block_index: bi,
            });
        }
    }
    chunks
}

/// In-memory keyword RAG index over document chunks.
#[derive(Debug, Clone, Default)]
pub struct DocumentRagIndex {
    chunks: Vec<DocumentChunk>,
}

impl DocumentRagIndex {
    pub fn from_document(doc: &Document) -> Self {
        Self {
            chunks: chunk_document(doc),
        }
    }

    pub fn from_chunks(chunks: Vec<DocumentChunk>) -> Self {
        Self { chunks }
    }

    pub fn chunks(&self) -> &[DocumentChunk] {
        &self.chunks
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Rank chunks by case-insensitive keyword overlap with [query].
    pub fn retrieve(&self, query: &str, top_k: usize) -> Vec<&DocumentChunk> {
        if top_k == 0 || self.chunks.is_empty() {
            return Vec::new();
        }
        let terms = tokenize(query);
        if terms.is_empty() {
            return self.chunks.iter().take(top_k).collect();
        }
        let mut scored: Vec<(usize, u32)> = self
            .chunks
            .iter()
            .enumerate()
            .map(|(i, chunk)| {
                let hay = chunk.text.to_lowercase();
                let score = terms
                    .iter()
                    .filter(|t| hay.contains(t.as_str()))
                    .count() as u32;
                (i, score)
            })
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        scored
            .into_iter()
            .filter(|(_, s)| *s > 0)
            .take(top_k)
            .map(|(i, _)| &self.chunks[i])
            .collect()
    }
}

/// Document-scoped chat session with RAG citations.
pub struct DocumentChatSession {
    index: DocumentRagIndex,
    history: Vec<ChatMessage>,
    pub top_k: usize,
}

impl DocumentChatSession {
    pub fn new(index: DocumentRagIndex) -> Self {
        Self {
            index,
            history: Vec::new(),
            top_k: 3,
        }
    }

    pub fn from_document(doc: &Document) -> Self {
        Self::new(DocumentRagIndex::from_document(doc))
    }

    pub fn index(&self) -> &DocumentRagIndex {
        &self.index
    }

    pub fn history(&self) -> &[ChatMessage] {
        &self.history
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Ask a question; retrieves chunks, calls the model, returns an assistant
    /// message with paragraph citations from retrieved context.
    pub fn ask(
        &mut self,
        router: &ProviderRouter,
        question: &str,
    ) -> Result<ChatMessage, AiError> {
        let retrieved = self.index.retrieve(question, self.top_k);
        let references: Vec<DocumentReference> = retrieved
            .iter()
            .map(|c| DocumentReference {
                paragraph_id: c.paragraph_id,
                excerpt: excerpt(&c.text, 120),
            })
            .collect();

        let context_block = if retrieved.is_empty() {
            "(no matching paragraphs)".to_string()
        } else {
            retrieved
                .iter()
                .map(|c| format!("[paragraph:{}] {}", c.paragraph_id, c.text))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let mut prompt = String::new();
        for msg in &self.history {
            let role = match msg.role {
                ChatRole::User => "User",
                ChatRole::Assistant => "Assistant",
                ChatRole::System => "System",
            };
            prompt.push_str(&format!("{role}: {}\n", msg.content));
        }
        prompt.push_str(&format!(
            "Context:\n{context_block}\n\nUser: {question}\nAssistant:"
        ));

        let token_estimate = retrieved
            .iter()
            .map(|c| c.token_estimate())
            .sum::<u32>()
            .saturating_add(question.len() as u32 / 4)
            .max(1);
        let ctx = DocumentContext {
            total_token_estimate: token_estimate,
            page_count: 1,
            ..Default::default()
        };

        let response = router.complete(
            AiTask::Chat,
            &ctx,
            &CompletionRequest {
                prompt,
                system_prompt: Some(
                    "You are a document assistant. Answer using the provided paragraphs. \
                     Cite paragraph ids when relevant."
                        .into(),
                ),
                max_tokens: 512,
                temperature: 0.3,
            },
        )?;

        let mut content = response.text;
        let parsed_ids = parse_cite_markers(&content);
        let refs = if parsed_ids.is_empty() {
            references
        } else {
            parsed_ids
                .into_iter()
                .filter_map(|id| {
                    self.index
                        .chunks()
                        .iter()
                        .find(|c| c.paragraph_id.to_string() == id)
                        .map(|c| DocumentReference {
                            paragraph_id: c.paragraph_id,
                            excerpt: excerpt(&c.text, 120),
                        })
                })
                .collect()
        };
        content = strip_cite_markers(&content);

        self.history.push(ChatMessage {
            role: ChatRole::User,
            content: question.to_string(),
            references: Vec::new(),
        });
        let assistant = ChatMessage {
            role: ChatRole::Assistant,
            content,
            references: refs,
        };
        self.history.push(assistant.clone());
        Ok(assistant)
    }
}

fn tokenize(query: &str) -> Vec<String> {
    query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() > 2)
        .map(|t| t.to_string())
        .collect()
}

fn excerpt(text: &str, max_chars: usize) -> String {
    let mut out: String = text.chars().take(max_chars).collect();
    if text.chars().count() > max_chars {
        out.push('…');
    }
    out
}

fn parse_cite_markers(text: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("[[cite:") {
        let after = &rest[start + 7..];
        if let Some(end) = after.find("]]") {
            ids.push(after[..end].to_string());
            rest = &after[end + 2..];
        } else {
            break;
        }
    }
    ids
}

fn strip_cite_markers(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("[[cite:") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 7..];
        if let Some(end) = after.find("]]") {
            rest = &after[end + 2..];
        } else {
            out.push_str("[[cite:");
            rest = after;
            break;
        }
    }
    out.push_str(rest);
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}
