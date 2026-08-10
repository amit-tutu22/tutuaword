import 'dart:convert';

import 'package:tutuaword/bridge/ai_client.dart';

/// Visual suggestion kinds (F28.S5).
enum AiVisualKindType { table, diagram, timeline }

class AiVisualKind {
  const AiVisualKind.table({this.rows = 3, this.cols = 3})
      : type = AiVisualKindType.table,
        diagramType = 0,
        stages = const [];

  const AiVisualKind.diagram({this.diagramType = 0})
      : type = AiVisualKindType.diagram,
        rows = 0,
        cols = 0,
        stages = const [];

  const AiVisualKind.timeline({required this.stages})
      : type = AiVisualKindType.timeline,
        rows = 1,
        cols = 0,
        diagramType = 0;

  final AiVisualKindType type;
  final int rows;
  final int cols;
  final int diagramType; // matches DiagramKind / smartArt*
  final List<String> stages;

  String get label => type.name;
}

class AiVisualSuggestion {
  const AiVisualSuggestion({
    required this.kind,
    required this.title,
    this.rationale = '',
  });

  final AiVisualKind kind;
  final String title;
  final String rationale;

  static String promptInstruction(String topic) =>
      'Suggest one visual for: $topic. Reply with ONLY JSON, no markdown fences. '
      'Schema examples: '
      '{"kind":"table","rows":3,"cols":3,"title":"…","rationale":"…"} '
      '{"kind":"diagram","diagram":"process|hierarchy|cycle","title":"…","rationale":"…"} '
      '{"kind":"timeline","stages":["A","B"],"title":"…","rationale":"…"}';
}

AiVisualSuggestion parseVisualSuggestion(String text) {
  final start = text.indexOf('{');
  final end = text.lastIndexOf('}');
  if (start < 0 || end <= start) {
    throw FormatException('visual suggestion is not valid JSON');
  }
  final map = jsonDecode(text.substring(start, end + 1)) as Map<String, dynamic>;
  final kindStr = (map['kind'] as String? ?? '').toLowerCase();
  final title = map['title'] as String? ?? 'Suggested visual';
  final rationale = map['rationale'] as String? ?? '';

  late final AiVisualKind kind;
  switch (kindStr) {
    case 'table':
      kind = AiVisualKind.table(
        rows: ((map['rows'] as num?)?.toInt() ?? 3).clamp(1, 63),
        cols: ((map['cols'] as num?)?.toInt() ?? 3).clamp(1, 63),
      );
    case 'diagram':
      final diagram = (map['diagram'] as String? ?? 'process').toLowerCase();
      final diagramType = switch (diagram) {
        'hierarchy' || 'org' => 1,
        'cycle' => 2,
        _ => 0,
      };
      kind = AiVisualKind.diagram(diagramType: diagramType);
    case 'timeline':
      final raw = map['stages'];
      var stages = <String>[];
      if (raw is List) {
        stages = raw.map((e) => e.toString()).toList();
      }
      if (stages.isEmpty) stages = ['Start', 'Finish'];
      kind = AiVisualKind.timeline(stages: stages);
    default:
      throw FormatException('unknown visual kind: $kindStr');
  }

  return AiVisualSuggestion(kind: kind, title: title, rationale: rationale);
}

Future<AiVisualSuggestion> suggestVisual({
  required AiClient client,
  required String topic,
}) async {
  final text = await client.generate(
    const AiDocumentContext(pageCount: 1, totalTokenEstimate: 100),
    AiVisualSuggestion.promptInstruction(topic),
  );
  return parseVisualSuggestion(text);
}
