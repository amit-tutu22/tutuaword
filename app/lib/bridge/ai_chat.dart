import 'package:tutuaword/bridge/ai_client.dart';

/// Document chat + keyword RAG (F28.S3), mirroring `tw_ai::chat`.

enum AiChatRole { user, assistant, system }

class AiDocumentReference {
  const AiDocumentReference({
    required this.paragraphId,
    required this.excerpt,
  });

  final String paragraphId;
  final String excerpt;
}

class AiChatMessage {
  const AiChatMessage({
    required this.role,
    required this.content,
    this.references = const [],
  });

  final AiChatRole role;
  final String content;
  final List<AiDocumentReference> references;
}

class AiDocumentChunk {
  const AiDocumentChunk({
    required this.paragraphId,
    required this.text,
    this.sectionIndex = 0,
    this.blockIndex = 0,
  });

  final String paragraphId;
  final String text;
  final int sectionIndex;
  final int blockIndex;
}

/// Chunk plain document text by newline into paragraph-like units.
List<AiDocumentChunk> chunkDocumentText(String text) {
  final lines = text.split('\n');
  final chunks = <AiDocumentChunk>[];
  for (var i = 0; i < lines.length; i++) {
    final line = lines[i].trimRight();
    if (line.trim().isEmpty) continue;
    chunks.add(AiDocumentChunk(
      paragraphId: 'para-$i',
      text: line,
      blockIndex: i,
    ));
  }
  return chunks;
}

class AiDocumentRagIndex {
  AiDocumentRagIndex(this.chunks);

  factory AiDocumentRagIndex.fromText(String text) =>
      AiDocumentRagIndex(chunkDocumentText(text));

  final List<AiDocumentChunk> chunks;

  int get length => chunks.length;
  bool get isEmpty => chunks.isEmpty;

  List<AiDocumentChunk> retrieve(String query, int topK) {
    if (topK <= 0 || chunks.isEmpty) return const [];
    final terms = _tokenize(query);
    if (terms.isEmpty) return chunks.take(topK).toList();
    final scored = <(int, int)>[];
    for (var i = 0; i < chunks.length; i++) {
      final hay = chunks[i].text.toLowerCase();
      final score = terms.where(hay.contains).length;
      scored.add((i, score));
    }
    scored.sort((a, b) {
      final byScore = b.$2.compareTo(a.$2);
      return byScore != 0 ? byScore : a.$1.compareTo(b.$1);
    });
    return scored
        .where((e) => e.$2 > 0)
        .take(topK)
        .map((e) => chunks[e.$1])
        .toList();
  }
}

class AiDocumentChatSession {
  AiDocumentChatSession(this.index, {this.topK = 3});

  factory AiDocumentChatSession.fromText(String text) =>
      AiDocumentChatSession(AiDocumentRagIndex.fromText(text));

  final AiDocumentRagIndex index;
  int topK;
  final List<AiChatMessage> history = [];

  void clearHistory() => history.clear();

  /// Ask [question] using [client] completions; returns assistant message with citations.
  Future<AiChatMessage> ask(AiClient client, String question) async {
    final retrieved = index.retrieve(question, topK);
    final references = retrieved
        .map(
          (c) => AiDocumentReference(
            paragraphId: c.paragraphId,
            excerpt: _excerpt(c.text, 120),
          ),
        )
        .toList();

    final contextBlock = retrieved.isEmpty
        ? '(no matching paragraphs)'
        : retrieved
            .map((c) => '[paragraph:${c.paragraphId}] ${c.text}')
            .join('\n');

    final buf = StringBuffer();
    for (final msg in history) {
      final role = switch (msg.role) {
        AiChatRole.user => 'User',
        AiChatRole.assistant => 'Assistant',
        AiChatRole.system => 'System',
      };
      buf.writeln('$role: ${msg.content}');
    }
    buf.writeln('Context:\n$contextBlock\n');
    buf.writeln('User: $question\nAssistant:');

    final tokenEstimate = retrieved.fold<int>(
          0,
          (sum, c) => sum + (c.text.length ~/ 4).clamp(1, 100000),
        ) +
        (question.length ~/ 4).clamp(1, 100000);

    var content = await client.chatComplete(
      AiDocumentContext(
        selectionText: buf.toString(),
        pageCount: 1,
        totalTokenEstimate: tokenEstimate,
      ),
    );

    final parsed = _parseCiteMarkers(content);
    late final List<AiDocumentReference> refs;
    if (parsed.isEmpty) {
      refs = references;
    } else {
      refs = [];
      for (final id in parsed) {
        for (final chunk in index.chunks) {
          if (chunk.paragraphId == id) {
            refs.add(AiDocumentReference(
              paragraphId: chunk.paragraphId,
              excerpt: _excerpt(chunk.text, 120),
            ));
            break;
          }
        }
      }
    }

    content = _stripCiteMarkers(content);

    history.add(AiChatMessage(role: AiChatRole.user, content: question));
    final assistant = AiChatMessage(
      role: AiChatRole.assistant,
      content: content,
      references: refs,
    );
    history.add(assistant);
    return assistant;
  }
}

List<String> _tokenize(String query) {
  return query
      .toLowerCase()
      .split(RegExp(r'[^a-z0-9]+'))
      .where((t) => t.length > 2)
      .toList();
}

String _excerpt(String text, int maxChars) {
  if (text.length <= maxChars) return text;
  return '${text.substring(0, maxChars)}…';
}

List<String> _parseCiteMarkers(String text) {
  final ids = <String>[];
  final re = RegExp(r'\[\[cite:([^\]]+)\]\]');
  for (final m in re.allMatches(text)) {
    ids.add(m.group(1)!);
  }
  return ids;
}

String _stripCiteMarkers(String text) {
  return text
      .replaceAll(RegExp(r'\[\[cite:[^\]]+\]\]'), '')
      .replaceAll(RegExp(r'\s+'), ' ')
      .trim();
}
