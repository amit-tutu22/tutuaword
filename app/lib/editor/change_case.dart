/// Text case transforms for Home → Change Case.
enum ChangeCaseKind {
  sentence,
  lower,
  upper,
  capitalizeEachWord,
  toggle,
}

String transformChangeCase(String text, ChangeCaseKind kind) {
  if (text.isEmpty) return text;
  return switch (kind) {
    ChangeCaseKind.sentence => _sentenceCase(text),
    ChangeCaseKind.lower => text.toLowerCase(),
    ChangeCaseKind.upper => text.toUpperCase(),
    ChangeCaseKind.capitalizeEachWord => _capitalizeEachWord(text),
    ChangeCaseKind.toggle => _toggleCase(text),
  };
}

String _sentenceCase(String text) {
  final lower = text.toLowerCase();
  final match = RegExp(r'[a-z]').firstMatch(lower);
  if (match == null) return lower;
  final i = match.start;
  return lower.substring(0, i) +
      lower[i].toUpperCase() +
      lower.substring(i + 1);
}

String _capitalizeEachWord(String text) {
  return text.splitMapJoin(
    RegExp(r'\S+'),
    onMatch: (m) {
      final word = m.group(0)!;
      if (word.isEmpty) return word;
      return word[0].toUpperCase() + word.substring(1).toLowerCase();
    },
  );
}

String _toggleCase(String text) {
  final letters = text.replaceAll(RegExp(r'[^A-Za-z]'), '');
  if (letters.isEmpty) return text;
  if (letters == letters.toUpperCase()) {
    return text.toLowerCase();
  }
  if (letters == letters.toLowerCase()) {
    return _sentenceCase(text);
  }
  return text.toUpperCase();
}

const kChangeCaseMenuLabels = <(ChangeCaseKind, String)>[
  (ChangeCaseKind.sentence, 'Sentence case'),
  (ChangeCaseKind.lower, 'lowercase'),
  (ChangeCaseKind.upper, 'UPPERCASE'),
  (ChangeCaseKind.capitalizeEachWord, 'Capitalize Each Word'),
  (ChangeCaseKind.toggle, 'tOGGLE cASE'),
];
