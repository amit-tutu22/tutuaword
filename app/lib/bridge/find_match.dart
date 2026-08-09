/// A find hit returned from the document engine (F18.S1).
class FindMatch {
  const FindMatch({
    String? runId,
    String? startRunId,
    String? endRunId,
    required this.start,
    required this.end,
  })  : startRunId = startRunId ?? runId!,
        endRunId = endRunId ?? startRunId ?? runId!;

  final String startRunId;
  final String endRunId;
  final int start;
  final int end;

  /// Legacy alias for [startRunId].
  String get runId => startRunId;

  factory FindMatch.fromJson(Map<String, dynamic> json) {
    final startRun =
        json['start_run_id'] as String? ?? json['run_id'] as String? ?? '';
    final endRun = json['end_run_id'] as String? ?? startRun;
    return FindMatch(
      startRunId: startRun,
      endRunId: endRun,
      start: json['start'] as int,
      end: json['end'] as int,
    );
  }
}
