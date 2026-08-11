import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/document_inspect_finding.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Result of Inspect Document: which categories to remove (F22.S3).
class DocumentInspectRemoval {
  const DocumentInspectRemoval({
    this.comments = false,
    this.metadata = false,
    this.hiddenText = false,
  });

  final bool comments;
  final bool metadata;
  final bool hiddenText;

  bool get any => comments || metadata || hiddenText;
}

/// Dialog listing Document Inspector findings with remove checkboxes (F22.S3).
class DocumentInspectorDialog extends StatefulWidget {
  const DocumentInspectorDialog({
    super.key,
    required this.findings,
  });

  final List<DocumentInspectFinding> findings;

  static Future<DocumentInspectRemoval?> show(
    BuildContext context,
    List<DocumentInspectFinding> findings,
  ) {
    return showDialog<DocumentInspectRemoval>(
      context: context,
      barrierDismissible: true,
      builder: (context) => DocumentInspectorDialog(findings: findings),
    );
  }

  @override
  State<DocumentInspectorDialog> createState() =>
      _DocumentInspectorDialogState();
}

class _DocumentInspectorDialogState extends State<DocumentInspectorDialog> {
  late bool _comments;
  late bool _metadata;
  late bool _hiddenText;

  @override
  void initState() {
    super.initState();
    _comments = _has('comments');
    _metadata = _has('metadata');
    _hiddenText = _has('hidden_text');
  }

  bool _has(String category) =>
      widget.findings.any((f) => f.category == category && f.count > 0);

  DocumentInspectFinding? _finding(String category) {
    for (final f in widget.findings) {
      if (f.category == category) return f;
    }
    return null;
  }

  void _remove() {
    Navigator.pop(
      context,
      DocumentInspectRemoval(
        comments: _comments && _has('comments'),
        metadata: _metadata && _has('metadata'),
        hiddenText: _hiddenText && _has('hidden_text'),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final empty = widget.findings.isEmpty;
    return AlertDialog(
      key: const Key('document_inspector_dialog'),
      title: const Text('Inspect Document'),
      content: SizedBox(
        width: 400,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              empty
                  ? 'No comments, document properties, or hidden text were found.'
                  : 'Select what to remove from this document.',
              style: const TextStyle(fontSize: 13, color: WordTheme.ribbonText),
            ),
            if (!empty) ...[
              const SizedBox(height: 12),
              _categoryTile(
                key: const Key('inspect_comments'),
                category: 'comments',
                label: 'Comments',
                value: _comments,
                onChanged: (v) => setState(() => _comments = v ?? false),
              ),
              _categoryTile(
                key: const Key('inspect_metadata'),
                category: 'metadata',
                label: 'Document properties',
                value: _metadata,
                onChanged: (v) => setState(() => _metadata = v ?? false),
              ),
              _categoryTile(
                key: const Key('inspect_hidden_text'),
                category: 'hidden_text',
                label: 'Hidden text',
                value: _hiddenText,
                onChanged: (v) => setState(() => _hiddenText = v ?? false),
              ),
            ],
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
        if (!empty)
          FilledButton(
            key: const Key('document_inspector_remove'),
            onPressed: (_comments || _metadata || _hiddenText) ? _remove : null,
            child: const Text('Remove'),
          ),
      ],
    );
  }

  Widget _categoryTile({
    required Key key,
    required String category,
    required String label,
    required bool value,
    required ValueChanged<bool?> onChanged,
  }) {
    final finding = _finding(category);
    final present = finding != null && finding.count > 0;
    return CheckboxListTile(
      key: key,
      contentPadding: EdgeInsets.zero,
      dense: true,
      controlAffinity: ListTileControlAffinity.leading,
      value: present && value,
      onChanged: present ? onChanged : null,
      title: Text(label, style: const TextStyle(fontSize: 13)),
      subtitle: Text(
        present ? finding.message : 'None found',
        style: TextStyle(
          fontSize: 11,
          color: present ? const Color(0xFF555555) : const Color(0xFF888888),
        ),
      ),
    );
  }
}
