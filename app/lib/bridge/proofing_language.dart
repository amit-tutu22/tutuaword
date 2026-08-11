/// Proofing / UI language options for Review → Language.
class ProofingLanguage {
  const ProofingLanguage({
    required this.id,
    required this.label,
    required this.translateName,
  });

  /// BCP-47 style id (e.g. `en-US`).
  final String id;

  /// Display label in the Language dialog.
  final String label;

  /// Name passed to AI translate prompts (e.g. `Spanish`).
  final String translateName;
}

const kDefaultProofingLanguageId = 'en-US';

const List<ProofingLanguage> kProofingLanguages = [
  ProofingLanguage(id: 'en-US', label: 'English (United States)', translateName: 'English'),
  ProofingLanguage(id: 'en-GB', label: 'English (United Kingdom)', translateName: 'English'),
  ProofingLanguage(id: 'es', label: 'Spanish', translateName: 'Spanish'),
  ProofingLanguage(id: 'fr', label: 'French', translateName: 'French'),
  ProofingLanguage(id: 'de', label: 'German', translateName: 'German'),
  ProofingLanguage(id: 'hi', label: 'Hindi', translateName: 'Hindi'),
  ProofingLanguage(id: 'pt', label: 'Portuguese', translateName: 'Portuguese'),
  ProofingLanguage(id: 'it', label: 'Italian', translateName: 'Italian'),
  ProofingLanguage(id: 'ja', label: 'Japanese', translateName: 'Japanese'),
  ProofingLanguage(id: 'zh', label: 'Chinese (Simplified)', translateName: 'Simplified Chinese'),
];

ProofingLanguage proofingLanguageById(String id) {
  return kProofingLanguages.firstWhere(
    (l) => l.id == id,
    orElse: () => kProofingLanguages.first,
  );
}
