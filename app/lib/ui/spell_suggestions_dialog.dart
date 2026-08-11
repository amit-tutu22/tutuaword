import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/spell_issue.dart';

/// Shows spell-check issues with suggestion chips (F17.S1 UX).
class SpellSuggestionsDialog extends StatelessWidget {
  const SpellSuggestionsDialog({
    super.key,
    required this.issues,
    this.onApplySuggestion,
  });

  final List<SpellIssue> issues;
  final Future<void> Function(SpellIssue issue, String suggestion)?
      onApplySuggestion;

  static Future<void> show(
    BuildContext context,
    List<SpellIssue> issues, {
    Future<void> Function(SpellIssue issue, String suggestion)?
        onApplySuggestion,
  }) {
    if (issues.isEmpty) return Future.value();
    return showDialog<void>(
      context: context,
      builder: (context) => SpellSuggestionsDialog(
        issues: issues,
        onApplySuggestion: onApplySuggestion,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Spelling suggestions'),
      content: SizedBox(
        width: 420,
        child: ListView.separated(
          shrinkWrap: true,
          itemCount: issues.length,
          separatorBuilder: (_, __) => const Divider(height: 16),
          itemBuilder: (context, index) {
            final issue = issues[index];
            return Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(issue.word, style: Theme.of(context).textTheme.titleSmall),
                if (issue.suggestions.isEmpty)
                  const Padding(
                    padding: EdgeInsets.only(top: 4),
                    child: Text('No suggestions'),
                  )
                else
                  Wrap(
                    spacing: 6,
                    runSpacing: 6,
                    children: issue.suggestions
                        .map(
                          (s) => ActionChip(
                            key: Key('spell_suggestion_${issue.word}_$s'),
                            label: Text(s),
                            onPressed: () async {
                              if (onApplySuggestion != null) {
                                await onApplySuggestion!(issue, s);
                              }
                              if (context.mounted) {
                                Navigator.of(context).pop();
                              }
                            },
                          ),
                        )
                        .toList(),
                  ),
              ],
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
