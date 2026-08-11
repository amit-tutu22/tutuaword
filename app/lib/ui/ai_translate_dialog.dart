import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/proofing_language.dart';

/// Pick a target language, then preview / accept an AI translation.
class AiTranslateDialog extends StatefulWidget {
  const AiTranslateDialog({
    super.key,
    required this.original,
    required this.suggestion,
    required this.targetLanguageId,
    this.busy = false,
  });

  final String original;
  final String suggestion;
  final String targetLanguageId;
  final bool busy;

  /// Phase 1: choose a target language. Returns language id or null.
  static Future<String?> pickLanguage(
    BuildContext context, {
    required String initialLanguageId,
  }) {
    return showDialog<String>(
      context: context,
      builder: (context) => _TranslateLanguagePicker(
        initialLanguageId: initialLanguageId,
      ),
    );
  }

  /// Phase 2: accept or discard the translation.
  static Future<String?> showResult(
    BuildContext context, {
    required String original,
    required String suggestion,
    required String targetLanguageId,
  }) {
    return showDialog<String>(
      context: context,
      barrierDismissible: true,
      builder: (context) => AiTranslateDialog(
        original: original,
        suggestion: suggestion,
        targetLanguageId: targetLanguageId,
      ),
    );
  }

  @override
  State<AiTranslateDialog> createState() => _AiTranslateDialogState();
}

class _AiTranslateDialogState extends State<AiTranslateDialog> {
  late final TextEditingController _suggestion;

  @override
  void initState() {
    super.initState();
    _suggestion = TextEditingController(text: widget.suggestion);
  }

  @override
  void dispose() {
    _suggestion.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final lang = proofingLanguageById(widget.targetLanguageId);
    return AlertDialog(
      key: const Key('ai_translate_dialog'),
      title: const Text('Translate selection'),
      content: SizedBox(
        width: 440,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              'Target: ${lang.label}',
              key: const Key('ai_translate_target'),
            ),
            const SizedBox(height: 8),
            const Text('Original', style: TextStyle(fontWeight: FontWeight.w600)),
            const SizedBox(height: 4),
            Container(
              key: const Key('ai_translate_original'),
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                border: Border.all(color: Theme.of(context).dividerColor),
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(widget.original),
            ),
            const SizedBox(height: 12),
            const Text('Translation', style: TextStyle(fontWeight: FontWeight.w600)),
            const SizedBox(height: 4),
            TextField(
              key: const Key('ai_translate_suggestion'),
              controller: _suggestion,
              maxLines: 4,
              decoration: const InputDecoration(border: OutlineInputBorder()),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('ai_translate_discard'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Discard'),
        ),
        FilledButton(
          key: const Key('ai_translate_accept'),
          onPressed: () => Navigator.of(context).pop(_suggestion.text),
          child: const Text('Accept'),
        ),
      ],
    );
  }
}

class _TranslateLanguagePicker extends StatefulWidget {
  const _TranslateLanguagePicker({required this.initialLanguageId});

  final String initialLanguageId;

  @override
  State<_TranslateLanguagePicker> createState() =>
      _TranslateLanguagePickerState();
}

class _TranslateLanguagePickerState extends State<_TranslateLanguagePicker> {
  late String _id;

  @override
  void initState() {
    super.initState();
    _id = widget.initialLanguageId;
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('ai_translate_language_picker'),
      title: const Text('Translate to'),
      content: SizedBox(
        width: 360,
        height: 320,
        child: ListView(
          key: const Key('ai_translate_language_list'),
          children: [
            for (final lang in kProofingLanguages)
              RadioListTile<String>(
                key: Key('ai_translate_lang_${lang.id}'),
                title: Text(lang.label),
                value: lang.id,
                groupValue: _id,
                onChanged: (v) {
                  if (v != null) setState(() => _id = v);
                },
              ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('ai_translate_language_cancel'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('ai_translate_language_ok'),
          onPressed: () => Navigator.of(context).pop(_id),
          child: const Text('Translate'),
        ),
      ],
    );
  }
}
