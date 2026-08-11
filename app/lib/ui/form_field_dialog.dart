import 'package:flutter/material.dart';

/// Kind of legacy Word form field (F26.S1).
enum FormFieldDialogKind {
  plainText,
  checkbox,
}

/// Result of Insert Form Field (F26.S1).
class FormFieldDialogResult {
  const FormFieldDialogResult({
    required this.kind,
    this.name,
    this.initialValue,
    this.checked = false,
  });

  final FormFieldDialogKind kind;
  final String? name;
  final String? initialValue;
  final bool checked;

  String get kindWire =>
      kind == FormFieldDialogKind.checkbox ? 'checkbox' : 'text';

  String? get initialValueWire {
    if (kind == FormFieldDialogKind.checkbox) {
      return checked ? 'true' : 'false';
    }
    final text = initialValue?.trim() ?? '';
    return text.isEmpty ? null : text;
  }
}

/// Word-like Insert Form Field dialog.
class FormFieldDialog extends StatefulWidget {
  const FormFieldDialog({super.key});

  static Future<FormFieldDialogResult?> show(BuildContext context) {
    return showDialog<FormFieldDialogResult>(
      context: context,
      barrierDismissible: true,
      builder: (context) => const FormFieldDialog(),
    );
  }

  @override
  State<FormFieldDialog> createState() => _FormFieldDialogState();
}

class _FormFieldDialogState extends State<FormFieldDialog> {
  FormFieldDialogKind _kind = FormFieldDialogKind.plainText;
  late final TextEditingController _nameController;
  late final TextEditingController _textController;
  bool _checked = false;

  @override
  void initState() {
    super.initState();
    _nameController = TextEditingController();
    _textController = TextEditingController();
  }

  @override
  void dispose() {
    _nameController.dispose();
    _textController.dispose();
    super.dispose();
  }

  void _submit() {
    final name = _nameController.text.trim();
    Navigator.pop(
      context,
      FormFieldDialogResult(
        kind: _kind,
        name: name.isEmpty ? null : name,
        initialValue: _textController.text,
        checked: _checked,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('form_field_dialog'),
      title: const Text('Insert Form Field'),
      content: SizedBox(
        width: 360,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            SegmentedButton<FormFieldDialogKind>(
              key: const Key('form_field_kind_toggle'),
              segments: const [
                ButtonSegment(
                  value: FormFieldDialogKind.plainText,
                  label: Text('Text'),
                  icon: Icon(Icons.short_text),
                ),
                ButtonSegment(
                  value: FormFieldDialogKind.checkbox,
                  label: Text('Checkbox'),
                  icon: Icon(Icons.check_box_outlined),
                ),
              ],
              selected: {_kind},
              onSelectionChanged: (next) {
                setState(() => _kind = next.first);
              },
            ),
            const SizedBox(height: 12),
            TextField(
              key: const Key('form_field_name_field'),
              controller: _nameController,
              decoration: const InputDecoration(
                labelText: 'Name (optional)',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 12),
            if (_kind == FormFieldDialogKind.plainText)
              TextField(
                key: const Key('form_field_text_field'),
                controller: _textController,
                decoration: const InputDecoration(
                  labelText: 'Default text',
                  border: OutlineInputBorder(),
                ),
              )
            else
              CheckboxListTile(
                key: const Key('form_field_checked_toggle'),
                contentPadding: EdgeInsets.zero,
                title: const Text('Checked by default'),
                value: _checked,
                onChanged: (v) => setState(() => _checked = v ?? false),
              ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('form_field_cancel_button'),
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('form_field_insert_button'),
          onPressed: _submit,
          child: const Text('Insert'),
        ),
      ],
    );
  }
}
