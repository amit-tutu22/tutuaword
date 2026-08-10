/// One accessibility checker finding (F21.S4).
class AccessibilityIssue {
  const AccessibilityIssue({
    required this.rule,
    required this.severity,
    required this.message,
    required this.nodeId,
    this.runId,
  });

  /// `missing_alt` | `empty_heading` | `low_contrast`
  final String rule;
  /// `error` | `warning`
  final String severity;
  final String message;
  final String nodeId;
  final String? runId;

  bool get isError => severity == 'error';
  bool get isWarning => severity == 'warning';

  factory AccessibilityIssue.fromJson(Map<String, dynamic> json) {
    return AccessibilityIssue(
      rule: json['rule'] as String? ?? '',
      severity: json['severity'] as String? ?? 'warning',
      message: json['message'] as String? ?? '',
      nodeId: json['node_id'] as String? ?? '',
      runId: json['run_id'] as String?,
    );
  }
}
