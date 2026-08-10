import 'package:flutter/material.dart';

/// Values for Insert → Cover Page.
class CoverPageValues {
  const CoverPageValues({
    required this.title,
    this.subtitle = '',
    this.author = '',
  });

  final String title;
  final String subtitle;
  final String author;
}

/// Simple cover-page fields (title / subtitle / author).
class CoverPageDialog extends StatefulWidget {
  const CoverPageDialog({
    super.key,
    this.initialTitle = 'Document Title',
    this.initialSubtitle = 'Subtitle',
    this.initialAuthor = '',
  });

  final String initialTitle;
  final String initialSubtitle;
  final String initialAuthor;

  static Future<CoverPageValues?> show(
    BuildContext context, {
    String initialTitle = 'Document Title',
    String initialSubtitle = 'Subtitle',
    String initialAuthor = '',
  }) {
    return showDialog<CoverPageValues>(
      context: context,
      builder: (context) => CoverPageDialog(
        initialTitle: initialTitle,
        initialSubtitle: initialSubtitle,
        initialAuthor: initialAuthor,
      ),
    );
  }

  @override
  State<CoverPageDialog> createState() => _CoverPageDialogState();
}

class _CoverPageDialogState extends State<CoverPageDialog> {
  late final TextEditingController _title;
  late final TextEditingController _subtitle;
  late final TextEditingController _author;

  @override
  void initState() {
    super.initState();
    _title = TextEditingController(text: widget.initialTitle);
    _subtitle = TextEditingController(text: widget.initialSubtitle);
    _author = TextEditingController(text: widget.initialAuthor);
  }

  @override
  void dispose() {
    _title.dispose();
    _subtitle.dispose();
    _author.dispose();
    super.dispose();
  }

  void _submit() {
    final title = _title.text.trim();
    if (title.isEmpty) return;
    Navigator.of(context).pop(
      CoverPageValues(
        title: title,
        subtitle: _subtitle.text.trim(),
        author: _author.text.trim(),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('cover_page_dialog'),
      title: const Text('Insert Cover Page'),
      content: SizedBox(
        width: 380,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            TextField(
              key: const Key('cover_page_title'),
              controller: _title,
              autofocus: true,
              decoration: const InputDecoration(
                labelText: 'Title',
                border: OutlineInputBorder(),
              ),
              onSubmitted: (_) => _submit(),
            ),
            const SizedBox(height: 12),
            TextField(
              key: const Key('cover_page_subtitle'),
              controller: _subtitle,
              decoration: const InputDecoration(
                labelText: 'Subtitle',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              key: const Key('cover_page_author'),
              controller: _author,
              decoration: const InputDecoration(
                labelText: 'Author',
                border: OutlineInputBorder(),
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('cover_page_cancel'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('cover_page_ok'),
          onPressed: _submit,
          child: const Text('Insert'),
        ),
      ],
    );
  }
}
