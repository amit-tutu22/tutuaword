/// One navigable heading or outline-numbered paragraph.
class DocumentOutlineEntry {
  const DocumentOutlineEntry({
    required this.paragraphId,
    required this.level,
    required this.text,
    required this.runId,
    required this.page,
  });

  final String paragraphId;
  final int level;
  final String text;
  final String runId;
  final int page;

  factory DocumentOutlineEntry.fromJson(Map<String, dynamic> json) {
    return DocumentOutlineEntry(
      paragraphId: json['paragraph_id'] as String,
      level: (json['level'] as num).toInt(),
      text: json['text'] as String? ?? '',
      runId: json['run_id'] as String,
      page: (json['page'] as num?)?.toInt() ?? 0,
    );
  }
}
