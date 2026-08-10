/// One Document Inspector category finding (F22.S3).
class DocumentInspectFinding {
  const DocumentInspectFinding({
    required this.category,
    required this.count,
    required this.message,
  });

  /// `comments` | `metadata` | `hidden_text`
  final String category;
  final int count;
  final String message;

  factory DocumentInspectFinding.fromJson(Map<String, dynamic> json) {
    return DocumentInspectFinding(
      category: json['category'] as String? ?? '',
      count: (json['count'] as num?)?.toInt() ?? 0,
      message: json['message'] as String? ?? '',
    );
  }
}
