import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/outline_entry.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/outline_navigator.dart';
import 'package:tutuaword/editor/page_navigator.dart';

enum NavigationPaneTab { pages, outline }

/// Left navigation strip with page thumbnails (F19.S1) and outline tree (F19.S2).
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
  void initState() {
    super.initState();
    if (widget.controller.view.takePreferOutlineTab()) {
      _tab = NavigationPaneTab.outline;
    }
  }

  @override
  void didUpdateWidget(covariant NavigationPane oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.controller.view.takePreferOutlineTab() &&
        _tab != NavigationPaneTab.outline) {
      setState(() => _tab = NavigationPaneTab.outline);
    }
  }

  @override
  Widget build(BuildContext context) {
    final outlineWidth = _tab == NavigationPaneTab.outline ? 200.0 : 140.0;
    return AnimatedContainer(
      duration: const Duration(milliseconds: 120),
      width: outlineWidth,
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
                : OutlineNavigator(
                    entries: widget.controller.documentOutline,
                    selectedRunId: widget.controller.caretRunId,
                    currentPage: widget.currentPage,
                    onSelected: widget.onOutlineSelected,
                  ),
          ),
        ],
      ),
    );
  }
}
