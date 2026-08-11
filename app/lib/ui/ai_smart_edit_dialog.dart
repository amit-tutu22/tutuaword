import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/ai_client.dart';
import 'package:tutuaword/bridge/ai_smart_edit.dart';

/// Review smart-edit suggestions before apply (F28.S6).
class AiSmartEditDialog extends StatefulWidget {
  const AiSmartEditDialog({
    super.key,
    required this.client,
    required this.documentText,
  });

  final AiClient client;
  final String documentText;

  static Future<AiSmartEditPlan?> show(
    BuildContext context, {
    required AiClient client,
    required String documentText,
  }) {
    return showDialog<AiSmartEditPlan>(
      context: context,
      barrierDismissible: true,
      builder: (context) => AiSmartEditDialog(
        client: client,
        documentText: documentText,
      ),
    );
  }

  @override
  State<AiSmartEditDialog> createState() => _AiSmartEditDialogState();
}

class _AiSmartEditDialogState extends State<AiSmartEditDialog> {
  AiSmartEditPlan? _plan;
  bool _busy = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _plan = analyzeDocumentHeuristics(widget.documentText);
  }

  Future<void> _refreshWithAi() async {
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final plan = await suggestSmartEdit(
        client: widget.client,
        documentText: widget.documentText,
      );
      if (mounted) setState(() => _plan = plan);
    } catch (e) {
      if (mounted) setState(() => _error = e.toString());
    }
    if (mounted) setState(() => _busy = false);
  }

  @override
  Widget build(BuildContext context) {
    final plan = _plan ?? AiSmartEditPlan();
    return AlertDialog(
      key: const Key('ai_smart_edit_dialog'),
      title: const Text('Smart edit'),
      content: SizedBox(
        width: 460,
        height: 400,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Text(
              'Review suggestions before applying. Nothing is saved without Accept.',
            ),
            const SizedBox(height: 8),
            Align(
              alignment: Alignment.centerLeft,
              child: OutlinedButton(
                key: const Key('ai_smart_edit_ai_refresh'),
                onPressed: _busy ? null : _refreshWithAi,
                child: Text(_busy ? 'Analyzing…' : 'Refine with AI'),
              ),
            ),
            if (_error != null) ...[
              const SizedBox(height: 8),
              Text(
                _error!,
                key: const Key('ai_smart_edit_error'),
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ],
            if (plan.autoFormatNotes.isNotEmpty) ...[
              const SizedBox(height: 8),
              Text(
                key: const Key('ai_smart_edit_notes'),
                plan.autoFormatNotes.join('\n'),
              ),
            ],
            const SizedBox(height: 8),
            Text(
              key: const Key('ai_smart_edit_heading_count'),
              '${plan.headings.length} heading suggestion(s)',
              style: const TextStyle(fontWeight: FontWeight.w600),
            ),
            Expanded(
              child: ListView.builder(
                key: const Key('ai_smart_edit_heading_list'),
                itemCount: plan.headings.length,
                itemBuilder: (context, index) {
                  final h = plan.headings[index];
                  return ListTile(
                    key: Key('ai_smart_edit_heading_$index'),
                    dense: true,
                    title: Text(h.previewText),
                    subtitle: Text(h.styleName),
                  );
                },
              ),
            ),
            CheckboxListTile(
              key: const Key('ai_smart_edit_toc'),
              value: plan.insertToc,
              onChanged: (v) {
                setState(() {
                  _plan = AiSmartEditPlan(
                    headings: plan.headings,
                    insertToc: v ?? false,
                    autoFormatNotes: plan.autoFormatNotes,
                  );
                });
              },
              title: const Text('Insert table of contents draft'),
              controlAffinity: ListTileControlAffinity.leading,
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('ai_smart_edit_discard'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Discard'),
        ),
        FilledButton(
          key: const Key('ai_smart_edit_accept'),
          onPressed: plan.isEmpty ? null : () => Navigator.of(context).pop(plan),
          child: const Text('Accept'),
        ),
      ],
    );
  }
}
