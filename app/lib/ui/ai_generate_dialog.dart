import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/ai_client.dart';
import 'package:tutuaword/bridge/ai_generate.dart';

/// Generate outline / minutes / report preview (F28.S4).
///
/// Returns [AiGeneratedDocument] when the user accepts into a new document.
class AiGenerateDialog extends StatefulWidget {
  const AiGenerateDialog({
    super.key,
    required this.client,
    this.initialKind = AiContentKind.outline,
  });

  final AiClient client;
  final AiContentKind initialKind;

  static Future<AiGeneratedDocument?> show(
    BuildContext context, {
    required AiClient client,
    AiContentKind initialKind = AiContentKind.outline,
  }) {
    return showDialog<AiGeneratedDocument>(
      context: context,
      barrierDismissible: true,
      builder: (context) => AiGenerateDialog(
        client: client,
        initialKind: initialKind,
      ),
    );
  }

  @override
  State<AiGenerateDialog> createState() => _AiGenerateDialogState();
}

class _AiGenerateDialogState extends State<AiGenerateDialog> {
  late AiContentKind _kind;
  late final TextEditingController _topic;
  AiGeneratedDocument? _generated;
  bool _busy = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _kind = widget.initialKind;
    _topic = TextEditingController(text: 'Project kickoff');
  }

  @override
  void dispose() {
    _topic.dispose();
    super.dispose();
  }

  Future<void> _runGenerate() async {
    final topic = _topic.text.trim();
    if (topic.isEmpty || _busy) return;
    setState(() {
      _busy = true;
      _error = null;
      _generated = null;
    });
    try {
      final doc = await generateContent(
        client: widget.client,
        kind: _kind,
        topic: topic,
      );
      if (mounted) setState(() => _generated = doc);
    } catch (e) {
      if (!mounted) return;
      final raw = e.toString();
      setState(() {
        _error = raw.startsWith('Bad state: ')
            ? raw.substring('Bad state: '.length)
            : raw;
      });
    }
    if (mounted) setState(() => _busy = false);
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('ai_generate_dialog'),
      title: const Text('Generate document'),
      content: SizedBox(
        width: 480,
        height: 420,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Wrap(
              spacing: 8,
              children: [
                for (final kind in AiContentKind.values)
                  ChoiceChip(
                    key: Key('ai_generate_kind_${kind.name}'),
                    label: Text(kind.label),
                    selected: _kind == kind,
                    onSelected: _busy
                        ? null
                        : (selected) {
                            if (selected) setState(() => _kind = kind);
                          },
                  ),
              ],
            ),
            const SizedBox(height: 12),
            TextField(
              key: const Key('ai_generate_topic'),
              controller: _topic,
              enabled: !_busy,
              decoration: const InputDecoration(
                labelText: 'Topic',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 8),
            Align(
              alignment: Alignment.centerLeft,
              child: FilledButton(
                key: const Key('ai_generate_run'),
                onPressed: _busy ? null : _runGenerate,
                child: Text(_busy ? 'Generating…' : 'Generate'),
              ),
            ),
            if (_error != null) ...[
              const SizedBox(height: 8),
              Text(
                _error!,
                key: const Key('ai_generate_error'),
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ],
            const SizedBox(height: 8),
            Expanded(
              child: DecoratedBox(
                decoration: BoxDecoration(
                  border: Border.all(color: Theme.of(context).dividerColor),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: SingleChildScrollView(
                  padding: const EdgeInsets.all(8),
                  child: Text(
                    key: const Key('ai_generate_preview'),
                    _generated?.plainText ?? 'Preview appears here',
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('ai_generate_cancel'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('ai_generate_open_new'),
          onPressed: _generated == null
              ? null
              : () => Navigator.of(context).pop(_generated),
          child: const Text('Open as new document'),
        ),
      ],
    );
  }
}
