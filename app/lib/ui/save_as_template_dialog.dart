import 'package:flutter/material.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Name a user template before persisting under My Templates (F24.S3).
class SaveAsTemplateDialog extends StatefulWidget {
  const SaveAsTemplateDialog({super.key, this.initialTitle = 'My Template'});

  final String initialTitle;

  static Future<String?> show(
    BuildContext context, {
    String initialTitle = 'My Template',
  }) {
    return showDialog<String>(
      context: context,
      barrierDismissible: true,
      builder: (context) => SaveAsTemplateDialog(initialTitle: initialTitle),
    );
  }

  @override
  State<SaveAsTemplateDialog> createState() => _SaveAsTemplateDialogState();
}

class _SaveAsTemplateDialogState extends State<SaveAsTemplateDialog> {
  late final TextEditingController _titleController;
  String? _error;

  @override
  void initState() {
    super.initState();
    _titleController = TextEditingController(text: widget.initialTitle);
  }

  @override
  void dispose() {
    _titleController.dispose();
    super.dispose();
  }

  void _submit() {
    final title = _titleController.text.trim();
    if (title.isEmpty) {
      setState(() => _error = 'Enter a template name.');
      return;
    }
    Navigator.pop(context, title);
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('save_as_template_dialog'),
      title: const Text('Save as Template'),
      content: SizedBox(
        width: 360,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Save a copy of this document as a starter template. '
              'It appears under My Templates in New from Template.',
              style: WordTheme.ribbonGroupLabel,
            ),
            if (_error != null) ...[
              const SizedBox(height: 10),
              Text(
                _error!,
                key: const Key('save_as_template_error'),
                style: const TextStyle(fontSize: 12, color: Color(0xFFC42B1C)),
              ),
            ],
            const SizedBox(height: 14),
            TextField(
              key: const Key('save_as_template_name'),
              controller: _titleController,
              decoration: const InputDecoration(
                labelText: 'Template name',
                border: OutlineInputBorder(),
                isDense: true,
              ),
              onSubmitted: (_) => _submit(),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('save_as_template_cancel'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        TextButton(
          key: const Key('save_as_template_save'),
          onPressed: _submit,
          child: const Text('Save'),
        ),
      ],
    );
  }
}
