import 'package:flutter/material.dart';
import 'package:tutuaword/ui/word_theme.dart';

enum PasteSpecialMode {
  keepSourceFormatting,
  plainText,
}

/// Paste Special choice: keep HTML/DOCX formatting or insert plain text only.
class PasteSpecialDialog extends StatelessWidget {
  const PasteSpecialDialog({super.key, required this.hasFormattedContent});

  final bool hasFormattedContent;

  static Future<PasteSpecialMode?> show(
    BuildContext context, {
    required bool hasFormattedContent,
  }) {
    return showDialog<PasteSpecialMode>(
      context: context,
      builder: (context) => PasteSpecialDialog(hasFormattedContent: hasFormattedContent),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Paste Special'),
      content: SizedBox(
        width: 360,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              hasFormattedContent
                  ? 'Choose how to paste clipboard content.'
                  : 'Clipboard has plain text only.',
              style: WordTheme.ribbonLabel,
            ),
            const SizedBox(height: 12),
            if (hasFormattedContent)
              FilledButton(
                onPressed: () => Navigator.of(context).pop(PasteSpecialMode.keepSourceFormatting),
                child: const Text('Keep Source Formatting'),
              ),
            if (hasFormattedContent) const SizedBox(height: 8),
            OutlinedButton(
              onPressed: () => Navigator.of(context).pop(PasteSpecialMode.plainText),
              child: const Text('Unformatted Text'),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
      ],
    );
  }
}
