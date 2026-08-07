//! R2.2 exit gate: 10k random edits; run text == text_in_range == export plaintext.

use tw_edit::{
    text_in_range, Command, DocPosition, DocRange, EditError, EditSession,
};
use tw_model::{CharFormat, Document, NodeId};

const SEED: u64 = 0xA5B2_2200_0001;
const EDIT_COUNT: usize = 10_000;
const CHECK_EVERY: usize = 500;

#[derive(Clone, Copy)]
enum Op {
    Insert,
    Delete,
    Split,
    Format,
}

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn next_usize(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        (self.next_u64() as usize) % bound
    }

    fn pick_op(&mut self) -> Op {
        match self.next_usize(100) {
            0..=69 => Op::Insert,
            70..=84 => Op::Delete,
            85..=92 => Op::Split,
            _ => Op::Format,
        }
    }

    fn insert_char(&mut self) -> char {
        const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789 \n";
        let idx = self.next_usize(ALPHABET.len());
        ALPHABET[idx] as char
    }
}

fn first_run_id(session: &EditSession) -> NodeId {
    session
        .document
        .paragraph_at(0, 0)
        .expect("paragraph")
        .runs[0]
        .id
}

fn run_len(session: &EditSession, run_id: NodeId) -> usize {
    tw_edit::run_with_id(&session.document, run_id)
        .map(|r| r.text().chars().count())
        .unwrap_or(0)
}

fn document_plain_text(doc: &Document) -> String {
    doc.sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .filter_map(|b| b.paragraph())
        .map(|p| p.visible_text())
        .collect::<Vec<_>>()
        .join("\n")
}

fn expected_visible_text(session: &EditSession) -> String {
    session
        .document
        .sections
        .iter()
        .flat_map(|s| s.blocks.iter())
        .filter_map(|b| b.paragraph())
        .map(|p| p.visible_text())
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_storage_consistent(session: &EditSession) {
    for section in &session.document.sections {
        for block in &section.blocks {
            let Some(para) = block.paragraph() else {
                continue;
            };
            for run in &para.runs {
                let run_text = run.text();
                let len = run_text.chars().count();
                let via_helper = tw_edit::run_char_len_by_id(&session.document, run.id);
                assert_eq!(via_helper, len, "run_char_len mismatch for {:?}", run.id);

                if len > 0 {
                    let slice = tw_edit::run_slice_by_id(&session.document, run.id, 0..len);
                    assert_eq!(slice, run_text, "run_slice mismatch for {:?}", run.id);
                }
            }
        }
    }

    let visible = expected_visible_text(session);
    let exported = document_plain_text(&session.document);
    assert_eq!(exported, visible, "export plaintext mismatch");

    if !visible.is_empty() {
        let run_id = first_run_id(session);
        let end = run_len(session, run_id);
        let range_text = text_in_range(
            &session.document,
            &DocRange {
                start: DocPosition {
                    run_id,
                    char_offset: 0,
                },
                end: DocPosition {
                    run_id,
                    char_offset: end.max(1),
                },
            },
        )
        .expect("text_in_range");
        assert!(
            visible.contains(&range_text) || range_text.is_empty(),
            "text_in_range not contained in visible text"
        );
    }
}

fn apply_random_op(session: &mut EditSession, rng: &mut Rng) -> Result<(), EditError> {
    let run_id = first_run_id(session);
    let len = run_len(session, run_id);
    match rng.pick_op() {
        Op::Insert => {
            let offset = rng.next_usize(len + 1);
            session.apply(Command::InsertText {
                run_id,
                offset,
                text: rng.insert_char().to_string(),
            })?;
        }
        Op::Delete => {
            if len < 2 {
                return Ok(());
            }
            let start = rng.next_usize(len - 1);
            let end = (start + 1 + rng.next_usize(len - start - 1)).min(len);
            if start < end {
                session.apply(Command::DeleteRange {
                    run_id,
                    start,
                    end,
                })?;
            }
        }
        Op::Split => {
            let offset = rng.next_usize(len + 1);
            session.apply(Command::SplitParagraphAt { run_id, offset })?;
        }
        Op::Format => {
            if len == 0 {
                return Ok(());
            }
            let start = rng.next_usize(len);
            let end = (start + rng.next_usize(len - start)).max(start + 1).min(len);
            session.apply(Command::SetCharFormat {
                run_id,
                start,
                end,
                format: CharFormat {
                    bold: Some(rng.next_usize(2) == 0),
                    ..Default::default()
                },
                merge: true,
            })?;
        }
    }
    Ok(())
}

#[test]
fn r2_single_text_storage_10k_random_edits() {
    let mut session = EditSession::new();
    let mut rng = Rng::new(SEED);

    assert_storage_consistent(&session);

    for i in 0..EDIT_COUNT {
        apply_random_op(&mut session, &mut rng).expect("random edit");
        if (i + 1) % CHECK_EVERY == 0 || i + 1 == EDIT_COUNT {
            assert_storage_consistent(&session);
        }
    }
}
