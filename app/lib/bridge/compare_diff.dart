/// Line-level document compare (Review → Compare), mirrored from tw-model.
enum CompareLineKind { equal, insert, delete }

class CompareLineChange {
  const CompareLineChange({required this.kind, required this.text});

  final CompareLineKind kind;
  final String text;
}

class CompareDiffResult {
  const CompareDiffResult({
    required this.changes,
    required this.insertionCount,
    required this.deletionCount,
  });

  final List<CompareLineChange> changes;
  final int insertionCount;
  final int deletionCount;

  String get summary =>
      'insertions:$insertionCount deletions:$deletionCount';

  List<CompareLineChange> get noteworthy =>
      changes.where((c) => c.kind != CompareLineKind.equal).toList();
}

CompareDiffResult compareDocumentLines(String left, String right) {
  final leftLines = left.isEmpty ? <String>[] : left.split('\n');
  final rightLines = right.isEmpty ? <String>[] : right.split('\n');
  final ops = _diffLines(leftLines, rightLines);
  var insertions = 0;
  var deletions = 0;
  for (final change in ops) {
    switch (change.kind) {
      case CompareLineKind.insert:
        insertions++;
      case CompareLineKind.delete:
        deletions++;
      case CompareLineKind.equal:
        break;
    }
  }
  return CompareDiffResult(
    changes: ops,
    insertionCount: insertions,
    deletionCount: deletions,
  );
}

List<CompareLineChange> _diffLines(List<String> left, List<String> right) {
  final lcs = _longestCommonSubsequence(left, right);
  final out = <CompareLineChange>[];
  var li = 0;
  var ri = 0;
  for (final pair in lcs) {
    final lIdx = pair.$1;
    final rIdx = pair.$2;
    while (li < lIdx) {
      out.add(CompareLineChange(kind: CompareLineKind.delete, text: left[li]));
      li++;
    }
    while (ri < rIdx) {
      out.add(CompareLineChange(kind: CompareLineKind.insert, text: right[ri]));
      ri++;
    }
    out.add(CompareLineChange(kind: CompareLineKind.equal, text: left[lIdx]));
    li = lIdx + 1;
    ri = rIdx + 1;
  }
  while (li < left.length) {
    out.add(CompareLineChange(kind: CompareLineKind.delete, text: left[li]));
    li++;
  }
  while (ri < right.length) {
    out.add(CompareLineChange(kind: CompareLineKind.insert, text: right[ri]));
    ri++;
  }
  return out;
}

List<(int, int)> _longestCommonSubsequence(List<String> left, List<String> right) {
  final dp = List.generate(
    left.length + 1,
    (_) => List<int>.filled(right.length + 1, 0),
  );
  for (var i = 0; i < left.length; i++) {
    for (var j = 0; j < right.length; j++) {
      if (left[i] == right[j]) {
        dp[i + 1][j + 1] = dp[i][j] + 1;
      } else {
        dp[i + 1][j + 1] =
            dp[i + 1][j] > dp[i][j + 1] ? dp[i + 1][j] : dp[i][j + 1];
      }
    }
  }
  final pairs = <(int, int)>[];
  var i = left.length;
  var j = right.length;
  while (i > 0 && j > 0) {
    if (left[i - 1] == right[j - 1]) {
      pairs.add((i - 1, j - 1));
      i--;
      j--;
    } else if (dp[i - 1][j] >= dp[i][j - 1]) {
      i--;
    } else {
      j--;
    }
  }
  return pairs.reversed.toList();
}
