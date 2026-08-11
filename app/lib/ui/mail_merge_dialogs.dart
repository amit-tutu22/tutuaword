import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/mail_merge_csv.dart';

/// Pick a CSV string / paste for Start Mail Merge (F26.S2).
class StartMailMergeDialog extends StatefulWidget {
  const StartMailMergeDialog({super.key, this.initialCsv});

  final String? initialCsv;

  static Future<MailMergeDataSource?> show(
    BuildContext context, {
    String? initialCsv,
  }) {
    return showDialog<MailMergeDataSource>(
      context: context,
      barrierDismissible: true,
      builder: (context) => StartMailMergeDialog(initialCsv: initialCsv),
    );
  }

  @override
  State<StartMailMergeDialog> createState() => _StartMailMergeDialogState();
}

class _StartMailMergeDialogState extends State<StartMailMergeDialog> {
  late final TextEditingController _csvController;
  String? _error;

  @override
  void initState() {
    super.initState();
    _csvController = TextEditingController(
      text: widget.initialCsv ??
          'Name,City\nAda,Paris\nGrace,London\n',
    );
  }

  @override
  void dispose() {
    _csvController.dispose();
    super.dispose();
  }

  void _submit() {
    try {
      final data = parseMailMergeCsv(_csvController.text);
      if (data.rows.isEmpty) {
        setState(() => _error = 'CSV must include at least one data row');
        return;
      }
      Navigator.pop(context, data);
    } on FormatException catch (e) {
      setState(() => _error = e.message);
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('start_mail_merge_dialog'),
      title: const Text('Start Mail Merge'),
      content: SizedBox(
        width: 420,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Text('Paste a CSV data source (header row required):'),
            const SizedBox(height: 8),
            TextField(
              key: const Key('mail_merge_csv_field'),
              controller: _csvController,
              maxLines: 8,
              decoration: const InputDecoration(
                border: OutlineInputBorder(),
                hintText: 'Name,City\nAda,Paris',
              ),
            ),
            if (_error != null) ...[
              const SizedBox(height: 8),
              Text(
                _error!,
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ],
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('mail_merge_cancel_button'),
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('mail_merge_start_button'),
          onPressed: _submit,
          child: const Text('Use CSV'),
        ),
      ],
    );
  }
}

/// Insert a merge field by name (F26.S2).
class InsertMergeFieldDialog extends StatefulWidget {
  const InsertMergeFieldDialog({super.key, this.headers = const []});

  final List<String> headers;

  static Future<String?> show(
    BuildContext context, {
    List<String> headers = const [],
  }) {
    return showDialog<String>(
      context: context,
      barrierDismissible: true,
      builder: (context) => InsertMergeFieldDialog(headers: headers),
    );
  }

  @override
  State<InsertMergeFieldDialog> createState() => _InsertMergeFieldDialogState();
}

class _InsertMergeFieldDialogState extends State<InsertMergeFieldDialog> {
  late final TextEditingController _nameController;

  @override
  void initState() {
    super.initState();
    _nameController = TextEditingController(
      text: widget.headers.isNotEmpty ? widget.headers.first : 'Name',
    );
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  void _submit() {
    final name = _nameController.text.trim();
    if (name.isEmpty) return;
    Navigator.pop(context, name);
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('insert_merge_field_dialog'),
      title: const Text('Insert Merge Field'),
      content: SizedBox(
        width: 320,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (widget.headers.isNotEmpty)
              DropdownButtonFormField<String>(
                key: const Key('merge_field_header_dropdown'),
                initialValue: widget.headers.contains(_nameController.text)
                    ? _nameController.text
                    : widget.headers.first,
                items: widget.headers
                    .map(
                      (h) => DropdownMenuItem(value: h, child: Text(h)),
                    )
                    .toList(),
                onChanged: (v) {
                  if (v != null) {
                    _nameController.text = v;
                    setState(() {});
                  }
                },
                decoration: const InputDecoration(
                  labelText: 'Field',
                  border: OutlineInputBorder(),
                ),
              )
            else
              TextField(
                key: const Key('merge_field_name_field'),
                controller: _nameController,
                decoration: const InputDecoration(
                  labelText: 'Field name',
                  border: OutlineInputBorder(),
                ),
              ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('merge_field_insert_button'),
          onPressed: _submit,
          child: const Text('Insert'),
        ),
      ],
    );
  }
}
