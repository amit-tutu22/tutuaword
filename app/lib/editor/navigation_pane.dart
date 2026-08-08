import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/outline_entry.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/page_navigator.dart';

enum NavigationPaneTab { pages, outline }

/// Left navigation strip with page thumbnails and document outline (F05.S4 / F19).
class NavigationPane extends StatefulWidget {
  const NavigationPane({
    super.key,
    required this.controller,
    required this.currentPage,
    required this.onPageSelected,
    required this.onOutlineSelected,
  });

  final EditorController controller;
  final int currentPage;
  final ValueChanged<int> onPageSelected;
  final ValueChanged<DocumentOutlineEntry> onOutlineSelected;

  @override
  State<NavigationPane> createState() => _NavigationPaneState();
}

class _NavigationPaneState extends State<NavigationPane> {
  NavigationPaneTab _tab = NavigationPaneTab.pages;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 140,
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
            child: Row(
              children: [
                Expanded(
                  child: Tooltip(
                    message: 'Pages',
                    child: IconButton(
                      isSelected: _tab == NavigationPaneTab.pages,
                      icon: const Icon(Icons.description_outlined, size: 18),
                      onPressed: () => setState(() => _tab = NavigationPaneTab.pages),
                    ),
                  ),
                ),
                Expanded(
                  child: Tooltip(
                    message: 'Outline',
                    child: IconButton(
                      isSelected: _tab == NavigationPaneTab.outline,
                      icon: const Icon(Icons.format_list_bulleted, size: 18),
                      onPressed: () => setState(() => _tab = NavigationPaneTab.outline),
                    ),
                  ),
                ),
              ],
            ),
          ),
          Expanded(
            child: _tab == NavigationPaneTab.pages
                ? PageNavigator(
                    controller: widget.controller,
                    currentPage: widget.currentPage,
                    onPageSelected: widget.onPageSelected,
                  )
                : _OutlineList(
                    entries: widget.controller.documentOutline,
                    onSelected: widget.onOutlineSelected,
                  ),
          ),
        ],
      ),
    );
  }
}

class _OutlineList extends StatelessWidget {
  const _OutlineList({
    required this.entries,
    required this.onSelected,
  });

  final List<DocumentOutlineEntry> entries;
  final ValueChanged<DocumentOutlineEntry> onSelected;

  @override
  Widget build(BuildContext context) {
    if (entries.isEmpty) {
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

    return ListView.builder(
      padding: const EdgeInsets.fromLTRB(8, 0, 8, 8),
      itemCount: entries.length,
      itemBuilder: (context, index) {
        final entry = entries[index];
        return InkWell(
          onTap: () => onSelected(entry),
          child: Padding(
            padding: EdgeInsets.only(left: 8.0 + entry.level * 12.0, top: 6, bottom: 6, right: 4),
            child: Text(
              entry.text,
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(
                fontSize: 12,
                fontWeight: entry.level == 0 ? FontWeight.w600 : FontWeight.normal,
              ),
            ),
          ),
        );
      },
    );
  }
}
