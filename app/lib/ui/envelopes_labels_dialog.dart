import 'package:flutter/material.dart';

enum MailCreateKind { envelope, label }

/// Mailings → Create Envelope or Label (MVP).
class EnvelopesLabelsDialog extends StatefulWidget {
  const EnvelopesLabelsDialog({super.key, required this.kind});

  final MailCreateKind kind;

  static Future<EnvelopesLabelsResult?> show(
    BuildContext context, {
    required MailCreateKind kind,
  }) {
    return showDialog<EnvelopesLabelsResult>(
      context: context,
      builder: (context) => EnvelopesLabelsDialog(kind: kind),
    );
  }

  @override
  State<EnvelopesLabelsDialog> createState() => _EnvelopesLabelsDialogState();
}

class EnvelopesLabelsResult {
  const EnvelopesLabelsResult({
    required this.kind,
    required this.address,
    required this.pageWidth,
    required this.pageHeight,
  });

  final MailCreateKind kind;
  final String address;
  final double pageWidth;
  final double pageHeight;
}

class _EnvelopesLabelsDialogState extends State<EnvelopesLabelsDialog> {
  late final TextEditingController _address;

  @override
  void initState() {
    super.initState();
    _address = TextEditingController(text: 'Recipient Name\n123 Main St\nCity, ST 12345');
  }

  @override
  void dispose() {
    _address.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final isEnvelope = widget.kind == MailCreateKind.envelope;
    return AlertDialog(
      title: Text(isEnvelope ? 'Envelopes' : 'Labels'),
      content: SizedBox(
        width: 360,
        child: TextField(
          key: Key(isEnvelope ? 'envelope_address' : 'label_address'),
          controller: _address,
          maxLines: 6,
          decoration: InputDecoration(
            labelText: isEnvelope ? 'Delivery address' : 'Label text',
            border: const OutlineInputBorder(),
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: Key(isEnvelope ? 'create_envelope' : 'create_label'),
          onPressed: () {
            Navigator.of(context).pop(
              EnvelopesLabelsResult(
                kind: widget.kind,
                address: _address.text.trim(),
                pageWidth: isEnvelope ? 684 : 288,
                pageHeight: isEnvelope ? 297 : 432,
              ),
            );
          },
          child: Text(isEnvelope ? 'Create Envelope' : 'Create Label'),
        ),
      ],
    );
  }
}
