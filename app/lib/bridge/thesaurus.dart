/// Local synonym lookup for Review → Thesaurus (offline-capable).
///
/// Keys are lowercase lemmas; values are synonym lists (excluding the key).
const Map<String, List<String>> kBuiltinThesaurus = {
  'happy': ['glad', 'joyful', 'cheerful', 'pleased', 'delighted'],
  'sad': ['unhappy', 'sorrowful', 'downcast', 'melancholy', 'gloomy'],
  'big': ['large', 'huge', 'enormous', 'substantial', 'great'],
  'small': ['little', 'tiny', 'compact', 'slight', 'minor'],
  'good': ['excellent', 'fine', 'great', 'positive', 'favorable'],
  'bad': ['poor', 'awful', 'terrible', 'negative', 'inferior'],
  'fast': ['quick', 'rapid', 'swift', 'speedy', 'brisk'],
  'slow': ['sluggish', 'leisurely', 'unhurried', 'gradual', 'delayed'],
  'smart': ['clever', 'intelligent', 'bright', 'sharp', 'astute'],
  'help': ['assist', 'aid', 'support', 'guide', 'serve'],
  'important': ['significant', 'crucial', 'vital', 'essential', 'key'],
  'show': ['display', 'present', 'reveal', 'demonstrate', 'exhibit'],
  'start': ['begin', 'commence', 'initiate', 'launch', 'open'],
  'end': ['finish', 'conclude', 'close', 'complete', 'terminate'],
  'say': ['state', 'remark', 'mention', 'express', 'declare'],
  'think': ['believe', 'consider', 'reckon', 'suppose', 'reflect'],
  'use': ['utilize', 'employ', 'apply', 'operate', 'exercise'],
  'make': ['create', 'build', 'produce', 'form', 'construct'],
  'change': ['alter', 'modify', 'adjust', 'transform', 'revise'],
  'document': ['file', 'record', 'paper', 'manuscript', 'text'],
};

/// Normalize a selection into a single lookup word.
String thesaurusLemma(String raw) {
  final trimmed = raw.trim();
  if (trimmed.isEmpty) return '';
  final first = trimmed.split(RegExp(r'\s+')).first.toLowerCase();
  return RegExp(r"[a-z0-9']+").firstMatch(first)?.group(0) ?? '';
}

/// Offline synonyms for [word]; empty when unknown.
List<String> lookupThesaurus(String word) {
  final lemma = thesaurusLemma(word);
  if (lemma.isEmpty) return const [];
  final hits = kBuiltinThesaurus[lemma];
  if (hits == null) return const [];
  return List<String>.from(hits);
}

/// Parse an AI thesaurus response into synonym strings.
List<String> parseThesaurusAiResponse(String raw, {String exclude = ''}) {
  final lemma = thesaurusLemma(exclude);
  final matches = RegExp(r"[A-Za-z][A-Za-z\-']{1,40}")
      .allMatches(raw)
      .map((m) => m.group(0)!.toLowerCase())
      .where((w) => w != lemma && w.length > 1)
      .toSet()
      .toList();
  return matches.take(12).toList(growable: false);
}
