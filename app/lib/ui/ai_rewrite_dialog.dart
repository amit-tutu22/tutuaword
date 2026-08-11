import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/ai_client.dart';

/// Preview / accept AI rewrite suggestion (F28.S2).
class AiRewriteDialog extends StatefulWidget {
  const AiRewriteDialog({
    super.key,
    required this.original,
    required this.suggestion,
    this.tone = AiRewriteTone.formal,
  });

  final String original;
  final String suggestion;
  final AiRewriteTone tone;

  /// Returns accepted suggestion text, or null if discarded.
  static Future<String?> show(
    BuildContext context, {
    required String original,
    required String suggestion,
    AiRewriteTone tone = AiRewriteTone.formal,
  }) {
    return showDialog<String>(
      context: context,
      barrierDismissible: true,
      builder: (context) => AiRewriteDialog(
        original: original,
        suggestion: suggestion,
        tone: tone,
      ),
    );
  }

  @override
  State<AiRewriteDialog> createState() => _AiRewriteDialogState();
}

class _AiRewriteDialogState extends State<AiRewriteDialog> {
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
    return AlertDialog(
      key: const Key('ai_rewrite_dialog'),
      title: const Text('Rewrite selection'),
      content: SizedBox(
        width: 440,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              'Tone: ${widget.tone.name}',
              key: const Key('ai_rewrite_tone'),
            ),
            const SizedBox(height: 8),
            const Text('Original', style: TextStyle(fontWeight: FontWeight.w600)),
            const SizedBox(height: 4),
            Container(
              key: const Key('ai_rewrite_original'),
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                border: Border.all(color: Theme.of(context).dividerColor),
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(widget.original),
            ),
            const SizedBox(height: 12),
            const Text('Suggestion', style: TextStyle(fontWeight: FontWeight.w600)),
            const SizedBox(height: 4),
            TextField(
              key: const Key('ai_rewrite_suggestion'),
              controller: _suggestion,
              maxLines: 4,
              decoration: const InputDecoration(
                border: OutlineInputBorder(),
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('ai_rewrite_discard'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Discard'),
        ),
        FilledButton(
          key: const Key('ai_rewrite_accept'),
          onPressed: () => Navigator.of(context).pop(_suggestion.text),
          child: const Text('Accept'),
        ),
      ],
    );
  }
}
