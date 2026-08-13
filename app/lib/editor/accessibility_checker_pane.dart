import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/accessibility_issue.dart';
import 'package:tutuaword/editor/editor_controller.dart';

/// Right-side pane listing accessibility checker findings (F21.S4).
class AccessibilityCheckerPane extends StatelessWidget {
  const AccessibilityCheckerPane({
    super.key,
    required this.controller,
    this.expanded = false,
  });

  final EditorController controller;
  final bool expanded;

  @override
  Widget build(BuildContext context) {
    final issues = controller.accessibilityIssues;
    return Container(
      key: const Key('accessibility_checker_pane'),
      width: expanded ? double.infinity : 240,
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 10, 8, 6),
            child: Row(
              children: [
                const Expanded(
                  child: Text(
                    'Accessibility',
                    style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600),
                  ),
                ),
                Tooltip(
                  message: 'Close',
                  child: IconButton(
                    icon: const Icon(Icons.close, size: 18),
                    onPressed: controller.hideAccessibilityChecker,
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
                  ),
                ),
              ],
            ),
          ),
          const Divider(height: 1),
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 8, 12, 4),
            child: Text(
              issues.isEmpty
                  ? 'No issues found.'
                  : '${issues.length} issue${issues.length == 1 ? '' : 's'}',
              style: const TextStyle(fontSize: 11, color: Color(0xFF555555)),
            ),
          ),
          Expanded(
            child: ListView.builder(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              itemCount: issues.length,
              itemBuilder: (context, index) {
                final issue = issues[index];
                return _IssueTile(
                  issue: issue,
                  onTap: () => controller.focusAccessibilityIssue(issue),
                );
              },
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(8),
            child: TextButton.icon(
              key: const Key('accessibility_checker_recheck'),
              onPressed: controller.checkAccessibility,
              icon: const Icon(Icons.refresh, size: 16),
              label: const Text('Recheck'),
            ),
          ),
        ],
      ),
    );
  }
}

class _IssueTile extends StatelessWidget {
  const _IssueTile({required this.issue, required this.onTap});

  final AccessibilityIssue issue;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final icon = switch (issue.rule) {
      'missing_alt' => Icons.image_not_supported_outlined,
      'empty_heading' => Icons.title,
      'low_contrast' => Icons.contrast,
      _ => Icons.warning_amber_outlined,
    };
    final color = issue.isError ? const Color(0xFFC42B1C) : const Color(0xFF9A6700);
    return Material(
      color: Colors.transparent,
      child: InkWell(
        key: Key('accessibility_issue_${issue.rule}_${issue.nodeId}'),
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 8),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(icon, size: 16, color: color),
              const SizedBox(width: 8),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      _ruleLabel(issue.rule),
                      style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.w600,
                        color: color,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      issue.message,
                      style: const TextStyle(fontSize: 11, height: 1.3),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  String _ruleLabel(String rule) {
    return switch (rule) {
      'missing_alt' => 'Missing alt text',
      'empty_heading' => 'Empty heading',
      'low_contrast' => 'Low contrast',
      _ => rule,
    };
  }
}
