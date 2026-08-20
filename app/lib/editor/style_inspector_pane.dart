import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';

/// Right-side pane showing resolved style sources at the caret (F06.S4).
class StyleInspectorPane extends StatelessWidget {
  const StyleInspectorPane({
    super.key,
    required this.controller,
    this.expanded = false,
  });

  final EditorController controller;
  final bool expanded;

  @override
  Widget build(BuildContext context) {
    final summary = controller.styleInspectorSummary;
    return Container(
      width: expanded ? double.infinity : 200,
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (!expanded) ...[
            Padding(
              padding: const EdgeInsets.fromLTRB(12, 10, 8, 6),
              child: Row(
                children: [
                  const Expanded(
                    child: Text(
                      'Style Inspector',
                      style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600),
                    ),
                  ),
                  Tooltip(
                    message: 'Close',
                    child: IconButton(
                      icon: const Icon(Icons.close, size: 18),
                      onPressed: controller.toggleStyleInspector,
                      padding: EdgeInsets.zero,
                      constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
                    ),
                  ),
                ],
              ),
            ),
            const Divider(height: 1),
          ],
          Expanded(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(12),
              child: Text(
                summary.isEmpty ? 'Place the caret in text to inspect formatting.' : summary,
                style: const TextStyle(fontSize: 12, height: 1.4),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
