import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/outline_entry.dart';

/// One node in the collapsible outline tree (F19.S2).
class OutlineTreeNode {
  OutlineTreeNode(this.entry);

  final DocumentOutlineEntry entry;
  final List<OutlineTreeNode> children = [];
}

/// Build a nested tree from a flat outline ordered by document order.
///
/// A child is attached to the nearest preceding entry with a strictly smaller
/// outline level (Word `outlineLvl` semantics).
List<OutlineTreeNode> buildOutlineTree(List<DocumentOutlineEntry> entries) {
  final roots = <OutlineTreeNode>[];
  final stack = <OutlineTreeNode>[];
  for (final entry in entries) {
    final node = OutlineTreeNode(entry);
    while (stack.isNotEmpty && stack.last.entry.level >= entry.level) {
      stack.removeLast();
    }
    if (stack.isEmpty) {
      roots.add(node);
    } else {
      stack.last.children.add(node);
    }
    stack.add(node);
  }
  return roots;
}

/// Collapsible headings tree for the navigation pane (F19.S2).
class OutlineNavigator extends StatefulWidget {
  const OutlineNavigator({
    super.key,
    required this.entries,
    required this.onSelected,
    this.selectedRunId,
    this.currentPage = 0,
  });

  final List<DocumentOutlineEntry> entries;
  final ValueChanged<DocumentOutlineEntry> onSelected;
  final String? selectedRunId;
  final int currentPage;

  @override
  State<OutlineNavigator> createState() => _OutlineNavigatorState();
}

class _OutlineNavigatorState extends State<OutlineNavigator> {
  /// Paragraph ids whose children are collapsed.
  final Set<String> _collapsed = {};

  @override
  Widget build(BuildContext context) {
    if (widget.entries.isEmpty) {
      return const Center(
        child: Padding(
          padding: EdgeInsets.all(12),
          child: Text(
            'No headings or outline-numbered paragraphs.',
            textAlign: TextAlign.center,
            style: TextStyle(fontSize: 12, color: Colors.grey),
          ),
        ),
      );
    }

    final roots = buildOutlineTree(widget.entries);
    final rows = <Widget>[];
    for (final root in roots) {
      _appendRows(rows, root, depth: 0);
    }

    return ListView(
      key: const ValueKey('outline-navigator-list'),
      padding: const EdgeInsets.fromLTRB(4, 0, 4, 8),
      children: rows,
    );
  }

  void _appendRows(List<Widget> rows, OutlineTreeNode node, {required int depth}) {
    final entry = node.entry;
    final hasChildren = node.children.isNotEmpty;
    final collapsed = _collapsed.contains(entry.paragraphId);
    final selected = widget.selectedRunId != null &&
        (entry.runId == widget.selectedRunId ||
            entry.paragraphId == widget.selectedRunId);
    final onCurrentPage = entry.page == widget.currentPage;

    rows.add(
      _OutlineRow(
        key: ValueKey('outline-entry-${entry.paragraphId}'),
        entry: entry,
        depth: depth,
        hasChildren: hasChildren,
        collapsed: collapsed,
        selected: selected,
        onCurrentPage: onCurrentPage,
        onToggle: hasChildren
            ? () {
                setState(() {
                  if (collapsed) {
                    _collapsed.remove(entry.paragraphId);
                  } else {
                    _collapsed.add(entry.paragraphId);
                  }
                });
              }
            : null,
        onTap: () => widget.onSelected(entry),
      ),
    );

    if (hasChildren && !collapsed) {
      for (final child in node.children) {
        _appendRows(rows, child, depth: depth + 1);
      }
    }
  }
}

class _OutlineRow extends StatelessWidget {
  const _OutlineRow({
    super.key,
    required this.entry,
    required this.depth,
    required this.hasChildren,
    required this.collapsed,
    required this.selected,
    required this.onCurrentPage,
    required this.onTap,
    this.onToggle,
  });

  final DocumentOutlineEntry entry;
  final int depth;
  final bool hasChildren;
  final bool collapsed;
  final bool selected;
  final bool onCurrentPage;
  final VoidCallback onTap;
  final VoidCallback? onToggle;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final background = selected
        ? theme.colorScheme.primary.withValues(alpha: 0.14)
        : onCurrentPage
            ? theme.colorScheme.surfaceContainerHigh
            : Colors.transparent;

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 1),
      child: Material(
        color: background,
        borderRadius: BorderRadius.circular(4),
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(4),
          child: Padding(
            padding: EdgeInsets.only(left: 4.0 + depth * 12.0, right: 4, top: 4, bottom: 4),
            child: Row(
              children: [
                SizedBox(
                  width: 22,
                  height: 22,
                  child: hasChildren
                      ? IconButton(
                          padding: EdgeInsets.zero,
                          iconSize: 16,
                          tooltip: collapsed ? 'Expand' : 'Collapse',
                          onPressed: onToggle,
                          icon: Icon(
                            collapsed
                                ? Icons.chevron_right
                                : Icons.expand_more,
                          ),
                        )
                      : const SizedBox.shrink(),
                ),
                Expanded(
                  child: Text(
                    entry.text,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(
                      fontSize: entry.level == 0 ? 12.5 : 12,
                      fontWeight: entry.level == 0
                          ? FontWeight.w700
                          : entry.level == 1
                              ? FontWeight.w600
                              : FontWeight.normal,
                      color: selected
                          ? theme.colorScheme.primary
                          : theme.colorScheme.onSurface,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
