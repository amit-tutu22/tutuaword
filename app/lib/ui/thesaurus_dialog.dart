import 'package:flutter/material.dart';

/// Pick a synonym to replace the selected word (Review → Thesaurus).
class ThesaurusDialog extends StatelessWidget {
  const ThesaurusDialog({
    super.key,
    required this.word,
    required this.synonyms,
  });

  final String word;
  final List<String> synonyms;

  /// Returns the chosen synonym, or null if cancelled.
  static Future<String?> show(
    BuildContext context, {
    required String word,
    required List<String> synonyms,
  }) {
    return showDialog<String>(
      context: context,
      barrierDismissible: true,
      builder: (context) => ThesaurusDialog(word: word, synonyms: synonyms),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('thesaurus_dialog'),
      title: Text('Thesaurus: $word'),
      content: SizedBox(
        width: 360,
        height: 280,
        child: synonyms.isEmpty
            ? const Text(
                key: Key('thesaurus_empty'),
                'No synonyms found. Try a different word or configure AI.',
              )
            : ListView.builder(
                key: const Key('thesaurus_list'),
                itemCount: synonyms.length,
                itemBuilder: (context, index) {
                  final syn = synonyms[index];
                  return ListTile(
                    key: Key('thesaurus_item_$syn'),
                    title: Text(syn),
                    onTap: () => Navigator.of(context).pop(syn),
                  );
                },
              ),
      ),
      actions: [
        TextButton(
          key: const Key('thesaurus_close'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Close'),
        ),
      ],
    );
  }
}
