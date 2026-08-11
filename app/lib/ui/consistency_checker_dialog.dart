import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/ai_client.dart';

class ConsistencyFinding {
  ConsistencyFinding({
    required this.term,
    required this.variants,
    this.suggestion,
  });

  final String term;
  final List<String> variants;
  final String? suggestion;
}

/// Review-tab consistency checker (smart differentiator).
class ConsistencyCheckerDialog extends StatefulWidget {
  const ConsistencyCheckerDialog({
    super.key,
    required this.findings,
    required this.onApply,
  });

  final List<ConsistencyFinding> findings;
  final Future<void> Function(ConsistencyFinding finding) onApply;

  static Future<void> show({
    required BuildContext context,
    required List<ConsistencyFinding> findings,
    required Future<void> Function(ConsistencyFinding finding) onApply,
  }) {
    return showDialog<void>(
      context: context,
      builder: (context) => ConsistencyCheckerDialog(
        findings: findings,
        onApply: onApply,
      ),
    );
  }

  @override
  State<ConsistencyCheckerDialog> createState() =>
      _ConsistencyCheckerDialogState();
}

class _ConsistencyCheckerDialogState extends State<ConsistencyCheckerDialog> {
  final Set<int> _ignored = {};

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Consistency checker'),
      content: SizedBox(
        width: 460,
        child: widget.findings.isEmpty
            ? const Text('No inconsistent terms found.')
            : ListView.builder(
                shrinkWrap: true,
                itemCount: widget.findings.length,
                itemBuilder: (context, index) {
                  if (_ignored.contains(index)) return const SizedBox.shrink();
                  final finding = widget.findings[index];
                  return ListTile(
                    key: Key('consistency_finding_$index'),
                    title: Text(finding.term),
                    subtitle: Text('Variants: ${finding.variants.join(', ')}'),
                    trailing: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        TextButton(
                          onPressed: () => setState(() => _ignored.add(index)),
                          child: const Text('Ignore'),
                        ),
                        FilledButton(
                          key: Key('consistency_apply_$index'),
                          onPressed: finding.suggestion == null
                              ? null
                              : () async {
                                  await widget.onApply(finding);
                                  if (context.mounted) {
                                    setState(() => _ignored.add(index));
                                  }
                                },
                          child: const Text('Apply'),
                        ),
                      ],
                    ),
                  );
                },
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

List<ConsistencyFinding> scanConsistencyFindings(String text) {
  final tokens = <String, int>{};
  final variants = <String, Set<String>>{};
  for (final raw in text.split(RegExp(r'[^A-Za-z0-9]+'))) {
    if (raw.length < 3) continue;
    final key = raw.toLowerCase();
    tokens[key] = (tokens[key] ?? 0) + 1;
    variants.putIfAbsent(key, () => {}).add(raw);
  }
  final findings = <ConsistencyFinding>[];
  for (final entry in variants.entries) {
    if (entry.value.length < 2) continue;
    final sorted = entry.value.toList()..sort();
    findings.add(
      ConsistencyFinding(
        term: entry.key,
        variants: sorted,
        suggestion: sorted.first,
      ),
    );
  }
  return findings;
}

Future<List<ConsistencyFinding>> scanWithAi(
  AiClient client,
  String documentText,
) async {
  final local = scanConsistencyFindings(documentText);
  if (local.isNotEmpty) return local;
  try {
    final reply = await client.rewrite(
      AiDocumentContext(selectionText: documentText),
      AiRewriteTone.formal,
    );
    if (reply.trim().isEmpty) return local;
    return [
      ConsistencyFinding(
        term: 'Review',
        variants: const ['AI flag'],
        suggestion: reply.trim(),
      ),
    ];
  } catch (_) {
    return local;
  }
}
