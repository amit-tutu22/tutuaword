import 'package:tutuaword/bridge/ai_client.dart';

/// Content generation kinds (F28.S4).
enum AiContentKind {
  outline,
  minutes,
  report,
}

extension AiContentKindX on AiContentKind {
  String get label => switch (this) {
        AiContentKind.outline => 'Outline',
        AiContentKind.minutes => 'Minutes',
        AiContentKind.report => 'Report',
      };

  String promptInstruction(String topic) => switch (this) {
        AiContentKind.outline =>
          'Generate a hierarchical markdown outline for: $topic. '
              'Use # for title and ## for sections. Keep it concise.',
        AiContentKind.minutes =>
          'Generate meeting minutes in markdown for: $topic. '
              'Include # title, ## Attendees, ## Discussion, ## Action items.',
        AiContentKind.report =>
          'Generate a short report in markdown for: $topic. '
              'Include # title, ## Summary, ## Findings, ## Recommendations.',
      };
}

class AiGeneratedParagraph {
  const AiGeneratedParagraph({
    required this.text,
    this.styleName,
  });

  final String text;
  final String? styleName;
}

class AiGeneratedDocument {
  const AiGeneratedDocument({
    required this.kind,
    required this.topic,
    required this.markdown,
    required this.paragraphs,
  });

  final AiContentKind kind;
  final String topic;
  final String markdown;
  final List<AiGeneratedParagraph> paragraphs;

  int get paragraphCount => paragraphs.length;

  String get plainText => paragraphs.map((p) => p.text).join('\n');
}

/// Parse AI markdown into styled paragraph descriptors.
List<AiGeneratedParagraph> paragraphsFromMarkdown(String markdown) {
  final out = <AiGeneratedParagraph>[];
  for (final raw in markdown.split('\n')) {
    final line = raw.trimRight();
    if (line.trim().isEmpty) continue;
    out.add(_classifyLine(line));
  }
  if (out.isEmpty) {
    out.add(const AiGeneratedParagraph(text: ''));
  }
  return out;
}

AiGeneratedParagraph _classifyLine(String line) {
  final trimmed = line.trimLeft();
  if (trimmed.startsWith('### ')) {
    return AiGeneratedParagraph(
      text: trimmed.substring(4).trim(),
      styleName: 'Heading 3',
    );
  }
  if (trimmed.startsWith('## ')) {
    return AiGeneratedParagraph(
      text: trimmed.substring(3).trim(),
      styleName: 'Heading 2',
    );
  }
  if (trimmed.startsWith('# ')) {
    return AiGeneratedParagraph(
      text: trimmed.substring(2).trim(),
      styleName: 'Heading 1',
    );
  }
  if (trimmed.startsWith('- ') || trimmed.startsWith('* ')) {
    return AiGeneratedParagraph(text: '• ${trimmed.substring(2)}');
  }
  return AiGeneratedParagraph(text: trimmed);
}

Future<AiGeneratedDocument> generateContent({
  required AiClient client,
  required AiContentKind kind,
  required String topic,
}) async {
  final instruction = kind.promptInstruction(topic);
  final markdown = await client.generate(
    const AiDocumentContext(pageCount: 1, totalTokenEstimate: 100),
    instruction,
  );
  return AiGeneratedDocument(
    kind: kind,
    topic: topic,
    markdown: markdown,
    paragraphs: paragraphsFromMarkdown(markdown),
  );
}
