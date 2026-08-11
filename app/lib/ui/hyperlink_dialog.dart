import 'package:flutter/material.dart';

/// Result of Insert/Edit Hyperlink (F19.S3).
class HyperlinkDialogResult {
  const HyperlinkDialogResult({
    required this.url,
    required this.text,
    this.tooltip,
  });

  final String url;
  final String text;
  final String? tooltip;
}

/// Word-like Insert Hyperlink dialog.
class HyperlinkDialog extends StatefulWidget {
  const HyperlinkDialog({
    super.key,
    this.initialUrl = 'https://',
    this.initialText = '',
    this.initialTooltip,
  });

  final String initialUrl;
  final String initialText;
  final String? initialTooltip;

  static Future<HyperlinkDialogResult?> show(
    BuildContext context, {
    String initialUrl = 'https://',
    String initialText = '',
    String? initialTooltip,
  }) {
    return showDialog<HyperlinkDialogResult>(
      context: context,
      barrierDismissible: true,
      builder: (context) => HyperlinkDialog(
        initialUrl: initialUrl,
        initialText: initialText,
        initialTooltip: initialTooltip,
      ),
    );
  }

  @override
  State<HyperlinkDialog> createState() => _HyperlinkDialogState();
}

class _HyperlinkDialogState extends State<HyperlinkDialog> {
  late final TextEditingController _urlController;
  late final TextEditingController _textController;
  late final TextEditingController _tooltipController;

  @override
  void initState() {
    super.initState();
    _urlController = TextEditingController(text: widget.initialUrl);
    _textController = TextEditingController(text: widget.initialText);
    _tooltipController = TextEditingController(text: widget.initialTooltip ?? '');
  }

  @override
  void dispose() {
    _urlController.dispose();
    _textController.dispose();
    _tooltipController.dispose();
    super.dispose();
  }

  void _submit() {
    final url = _urlController.text.trim();
    if (url.isEmpty || url == 'https://' || url == 'http://') {
      return;
    }
    final text = _textController.text.trim().isEmpty
        ? url
        : _textController.text.trim();
    final tip = _tooltipController.text.trim();
    Navigator.pop(
      context,
      HyperlinkDialogResult(
        url: url,
        text: text,
        tooltip: tip.isEmpty ? null : tip,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('hyperlink_dialog'),
      title: const Text('Insert Hyperlink'),
      content: SizedBox(
        width: 380,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              key: const Key('hyperlink_text_field'),
              controller: _textController,
              decoration: const InputDecoration(
                labelText: 'Text to display',
              ),
              textInputAction: TextInputAction.next,
            ),
            const SizedBox(height: 12),
            TextField(
              key: const Key('hyperlink_url_field'),
              controller: _urlController,
              autofocus: true,
              decoration: const InputDecoration(
                labelText: 'Address',
                hintText: 'https://example.com',
              ),
              keyboardType: TextInputType.url,
              onSubmitted: (_) => _submit(),
            ),
            const SizedBox(height: 12),
            TextField(
              key: const Key('hyperlink_tooltip_field'),
              controller: _tooltipController,
              decoration: const InputDecoration(
                labelText: 'ScreenTip (optional)',
              ),
              onSubmitted: (_) => _submit(),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        TextButton(
          key: const Key('hyperlink_insert_button'),
          onPressed: _submit,
          child: const Text('Insert'),
        ),
      ],
    );
  }
}

/// Simple bookmark name dialog (F19.S3).
class BookmarkNameDialog extends StatefulWidget {
  const BookmarkNameDialog({super.key, this.initialName = 'SectionRef'});

  final String initialName;

  static Future<String?> show(
    BuildContext context, {
    String initialName = 'SectionRef',
  }) {
    return showDialog<String>(
      context: context,
      barrierDismissible: true,
      builder: (context) => BookmarkNameDialog(initialName: initialName),
    );
  }

  @override
  State<BookmarkNameDialog> createState() => _BookmarkNameDialogState();
}

class _BookmarkNameDialogState extends State<BookmarkNameDialog> {
  late final TextEditingController _controller;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.initialName);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _submit() {
    final name = _controller.text.trim();
    if (name.isEmpty) return;
    Navigator.pop(context, name);
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('bookmark_name_dialog'),
      title: const Text('Bookmark'),
      content: SizedBox(
        width: 320,
        child: TextField(
          key: const Key('bookmark_name_field'),
          controller: _controller,
          autofocus: true,
          decoration: const InputDecoration(labelText: 'Bookmark name'),
          onSubmitted: (_) => _submit(),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        TextButton(
          key: const Key('bookmark_insert_button'),
          onPressed: _submit,
          child: const Text('Add'),
        ),
      ],
    );
  }
}
