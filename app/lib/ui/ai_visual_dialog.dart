import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/ai_client.dart';
import 'package:tutuaword/bridge/ai_visual.dart';

/// Suggest a table / diagram / timeline and accept insert (F28.S5).
class AiVisualDialog extends StatefulWidget {
  const AiVisualDialog({
    super.key,
    required this.client,
    this.initialTopic = '',
  });

  final AiClient client;
  final String initialTopic;

  static Future<AiVisualSuggestion?> show(
    BuildContext context, {
    required AiClient client,
    String initialTopic = '',
  }) {
    return showDialog<AiVisualSuggestion>(
      context: context,
      barrierDismissible: true,
      builder: (context) => AiVisualDialog(
        client: client,
        initialTopic: initialTopic,
      ),
    );
  }

  @override
  State<AiVisualDialog> createState() => _AiVisualDialogState();
}

class _AiVisualDialogState extends State<AiVisualDialog> {
  late final TextEditingController _topic;
  AiVisualSuggestion? _suggestion;
  bool _busy = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _topic = TextEditingController(
      text: widget.initialTopic.isEmpty ? 'project roadmap' : widget.initialTopic,
    );
  }

  @override
  void dispose() {
    _topic.dispose();
    super.dispose();
  }

  Future<void> _suggest() async {
    final topic = _topic.text.trim();
    if (topic.isEmpty || _busy) return;
    setState(() {
      _busy = true;
      _error = null;
      _suggestion = null;
    });
    try {
      final s = await suggestVisual(client: widget.client, topic: topic);
      if (mounted) setState(() => _suggestion = s);
    } catch (e) {
      if (mounted) setState(() => _error = e.toString());
    }
    if (mounted) setState(() => _busy = false);
  }

  String _detail(AiVisualSuggestion s) {
    return switch (s.kind.type) {
      AiVisualKindType.table =>
        'Table ${s.kind.rows}×${s.kind.cols}',
      AiVisualKindType.diagram =>
        'Diagram (${switch (s.kind.diagramType) {
          1 => 'hierarchy',
          2 => 'cycle',
          _ => 'process',
        }})',
      AiVisualKindType.timeline =>
        'Timeline: ${s.kind.stages.join(' → ')}',
    };
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('ai_visual_dialog'),
      title: const Text('Suggest visual'),
      content: SizedBox(
        width: 440,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            TextField(
              key: const Key('ai_visual_topic'),
              controller: _topic,
              enabled: !_busy,
              decoration: const InputDecoration(
                labelText: 'Topic / selection',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 8),
            Align(
              alignment: Alignment.centerLeft,
              child: FilledButton(
                key: const Key('ai_visual_suggest'),
                onPressed: _busy ? null : _suggest,
                child: Text(_busy ? 'Suggesting…' : 'Suggest'),
              ),
            ),
            if (_error != null) ...[
              const SizedBox(height: 8),
              Text(
                _error!,
                key: const Key('ai_visual_error'),
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ],
            if (_suggestion != null) ...[
              const SizedBox(height: 12),
              Text(
                key: const Key('ai_visual_title'),
                _suggestion!.title,
                style: const TextStyle(fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 4),
              Text(
                key: const Key('ai_visual_detail'),
                _detail(_suggestion!),
              ),
              if (_suggestion!.rationale.isNotEmpty) ...[
                const SizedBox(height: 4),
                Text(
                  key: const Key('ai_visual_rationale'),
                  _suggestion!.rationale,
                ),
              ],
            ],
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('ai_visual_discard'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Discard'),
        ),
        FilledButton(
          key: const Key('ai_visual_insert'),
          onPressed: _suggestion == null
              ? null
              : () => Navigator.of(context).pop(_suggestion),
          child: const Text('Insert'),
        ),
      ],
    );
  }
}
