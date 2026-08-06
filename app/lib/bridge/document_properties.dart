/// Document metadata from `docProps/core.xml` and layout.
class DocumentProperties {
  const DocumentProperties({
    this.title,
    this.author,
    this.pageCount,
  });

  final String? title;
  final String? author;
  final int? pageCount;

  static const empty = DocumentProperties();

  factory DocumentProperties.fromJson(Map<String, dynamic> json) {
    return DocumentProperties(
      title: json['title'] as String?,
      author: json['author'] as String?,
      pageCount: json['page_count'] as int?,
    );
  }

  String displayValue(String? value) => (value == null || value.isEmpty) ? '—' : value;
}
