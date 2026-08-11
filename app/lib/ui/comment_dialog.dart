import 'package:flutter/material.dart';

/// Prompt for Review → New Comment body text.
class CommentDialog extends StatefulWidget {
  const CommentDialog({super.key, this.initialText = ''});

  final String initialText;

  static Future<String?> show(BuildContext context, {String initialText = ''}) {
    return showDialog<String>(
      context: context,
      builder: (context) => CommentDialog(initialText: initialText),
    );
  }

  @override
  State<CommentDialog> createState() => _CommentDialogState();
}

class _CommentDialogState extends State<CommentDialog> {
  late final TextEditingController _body;

  @override
  void initState() {
    super.initState();
    _body = TextEditingController(text: widget.initialText);
  }

  @override
  void dispose() {
    _body.dispose();
    super.dispose();
  }

  void _submit() {
    final text = _body.text.trim();
    if (text.isEmpty) return;
    Navigator.of(context).pop(text);
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('comment_dialog'),
      title: const Text('New Comment'),
      content: SizedBox(
        width: 360,
        child: TextField(
          key: const Key('comment_body'),
          controller: _body,
          autofocus: true,
          maxLines: 4,
          decoration: const InputDecoration(
            labelText: 'Comment',
            border: OutlineInputBorder(),
          ),
          onSubmitted: (_) => _submit(),
        ),
      ),
      actions: [
        TextButton(
          key: const Key('comment_cancel'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('comment_ok'),
          onPressed: _submit,
          child: const Text('Insert'),
        ),
      ],
    );
  }
}
