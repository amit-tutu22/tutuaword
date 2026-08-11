import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/proofing_language.dart';

/// Choose the document proofing / default translate language.
class LanguageDialog extends StatefulWidget {
  const LanguageDialog({
    super.key,
    required this.currentLanguageId,
  });

  final String currentLanguageId;

  /// Returns the selected language id, or null if cancelled.
  static Future<String?> show(
    BuildContext context, {
    required String currentLanguageId,
  }) {
    return showDialog<String>(
      context: context,
      barrierDismissible: true,
      builder: (context) => LanguageDialog(currentLanguageId: currentLanguageId),
    );
  }

  @override
  State<LanguageDialog> createState() => _LanguageDialogState();
}

class _LanguageDialogState extends State<LanguageDialog> {
  late String _id;

  @override
  void initState() {
    super.initState();
    _id = widget.currentLanguageId;
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('language_dialog'),
      title: const Text('Set Proofing Language'),
      content: SizedBox(
        width: 360,
        height: 320,
        child: ListView(
          children: [
            for (final lang in kProofingLanguages)
              RadioListTile<String>(
                key: Key('language_option_${lang.id}'),
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
          key: const Key('language_cancel'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('language_ok'),
          onPressed: () => Navigator.of(context).pop(_id),
          child: const Text('OK'),
        ),
      ],
    );
  }
}
