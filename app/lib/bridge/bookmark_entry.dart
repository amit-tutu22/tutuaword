/// One navigable bookmark for Go To (F19.S4).
class DocumentBookmarkEntry {
  const DocumentBookmarkEntry({
    required this.name,
    required this.runId,
    required this.paragraphId,
    required this.page,
  });

  final String name;
  final String runId;
  final String paragraphId;
  final int page;

  factory DocumentBookmarkEntry.fromJson(Map<String, dynamic> json) {
    return DocumentBookmarkEntry(
      name: json['name'] as String? ?? '',
      runId: json['run_id'] as String? ?? '',
      paragraphId: json['paragraph_id'] as String? ?? '',
      page: (json['page'] as num?)?.toInt() ?? 0,
    );
  }
}
