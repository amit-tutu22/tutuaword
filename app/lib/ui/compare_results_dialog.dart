import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/compare_diff.dart';

/// Review → Compare results: line insertions / deletions vs another document.
class CompareResultsDialog extends StatelessWidget {
  const CompareResultsDialog({
    super.key,
    required this.result,
    required this.otherLabel,
  });

  final CompareDiffResult result;
  final String otherLabel;

  static Future<void> show(
    BuildContext context, {
    required CompareDiffResult result,
    required String otherLabel,
  }) {
    return showDialog<void>(
      context: context,
      builder: (context) => CompareResultsDialog(
        result: result,
        otherLabel: otherLabel,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final changes = result.noteworthy;
    return AlertDialog(
      key: const Key('compare_results_dialog'),
      title: const Text('Compare documents'),
      content: SizedBox(
        width: 480,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              key: const Key('compare_results_summary'),
              'Compared with “$otherLabel”\n'
              '${result.insertionCount} insertion(s), '
              '${result.deletionCount} deletion(s)',
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            const SizedBox(height: 12),
            if (changes.isEmpty)
              const Text(
                key: Key('compare_results_identical'),
                'Documents are identical (line-level).',
              )
            else
              SizedBox(
                height: 280,
                child: ListView.builder(
                  key: const Key('compare_results_list'),
                  itemCount: changes.length,
                  itemBuilder: (context, index) {
                    final change = changes[index];
                    final isInsert = change.kind == CompareLineKind.insert;
                    final color = isInsert
                        ? const Color(0xFF1B5E20)
                        : const Color(0xFFB71C1C);
                    final bg = isInsert
                        ? const Color(0xFFE8F5E9)
                        : const Color(0xFFFFEBEE);
                    final prefix = isInsert ? '+ ' : '− ';
                    return Container(
                      key: Key('compare_change_$index'),
                      width: double.infinity,
                      margin: const EdgeInsets.only(bottom: 4),
                      padding: const EdgeInsets.symmetric(
                        horizontal: 8,
                        vertical: 6,
                      ),
                      color: bg,
                      child: Text(
                        '$prefix${change.text}',
                        style: TextStyle(
                          color: color,
                          fontFamily: 'monospace',
                          fontSize: 13,
                        ),
                      ),
                    );
                  },
                ),
              ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('compare_results_close'),
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
      ],
    );
  }
}
