/// One node in the parallel accessibility tree (F21.S1).
class SemanticDocumentNode {
  const SemanticDocumentNode({
    required this.id,
    required this.role,
    required this.text,
    this.level,
    this.children = const [],
  });

  final String id;
  /// `heading` | `paragraph` | `table` | `table_cell`
  final String role;
  final String text;
  final int? level;
  final List<SemanticDocumentNode> children;

  factory SemanticDocumentNode.fromJson(Map<String, dynamic> json) {
    final rawChildren = json['children'];
    final children = rawChildren is List
        ? rawChildren
            .whereType<Map>()
            .map((c) => SemanticDocumentNode.fromJson(Map<String, dynamic>.from(c)))
            .toList()
        : const <SemanticDocumentNode>[];
    return SemanticDocumentNode(
      id: json['id'] as String? ?? '',
      role: json['role'] as String? ?? 'paragraph',
      text: json['text'] as String? ?? '',
      level: (json['level'] as num?)?.toInt(),
      children: children,
    );
  }
}
