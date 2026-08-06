import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// View-only document properties (title, author, page count).
class DocumentPropertiesDialog extends StatelessWidget {
  const DocumentPropertiesDialog({super.key, required this.properties});

  final DocumentProperties properties;

  static Future<void> show(BuildContext context, DocumentProperties properties) {
    return showDialog<void>(
      context: context,
      builder: (context) => DocumentPropertiesDialog(properties: properties),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Document Properties'),
      content: SizedBox(
        width: 360,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _PropertyRow(label: 'Title', value: properties.displayValue(properties.title)),
            _PropertyRow(label: 'Author', value: properties.displayValue(properties.author)),
            _PropertyRow(
              label: 'Pages',
              value: properties.pageCount?.toString() ?? '—',
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Close'),
        ),
      ],
    );
  }
}

class _PropertyRow extends StatelessWidget {
  const _PropertyRow({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 72,
            child: Text(label, style: WordTheme.ribbonLabel.copyWith(fontWeight: FontWeight.w600)),
          ),
          Expanded(
            child: SelectableText(value, style: WordTheme.ribbonLabel),
          ),
        ],
      ),
    );
  }
}
