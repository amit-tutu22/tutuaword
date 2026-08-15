import 'package:flutter/material.dart';
import 'package:tutuaword/editor/document_view_layout.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';
import 'package:tutuaword/ui/secondary_document_window.dart';
import 'package:tutuaword/ui/word_theme.dart';

class ViewTab extends StatelessWidget {
  const ViewTab({super.key, required this.controller});

  final EditorController controller;

  Future<void> _openNewWindow(BuildContext context) async {
    controller.markNewWindowOpened();
    await SecondaryDocumentWindow.show(context, controller: controller);
    controller.markNewWindowClosed();
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        final layout = controller.viewLayout;
        return RibbonTabScroller(
          children: [
              RibbonGroup(
                label: 'Views',
                child: Row(
                  children: [
                    RibbonLargeButton(
                      key: const Key('view_read_mode'),
                      icon: Icons.article_outlined,
                      label: 'Read\nMode',
                      tooltip: 'Read-only reading view',
                      onPressed: controller.setReadMode,
                    ),
                    RibbonLargeButton(
                      key: const Key('view_print_layout'),
                      icon: Icons.print_outlined,
                      label: 'Print\nLayout',
                      tooltip: layout == DocumentViewLayout.printLayout &&
                              !controller.printPreview
                          ? 'Print layout (current)'
                          : 'Print layout editing view',
                      onPressed: controller.setPrintLayout,
                    ),
                    RibbonLargeButton(
                      key: const Key('view_print_preview'),
                      icon: Icons.preview_outlined,
                      label: 'Print\nPreview',
                      tooltip: controller.printPreview
                          ? 'Print preview (current)'
                          : 'Read-only print preview',
                      onPressed: controller.setPrintPreviewMode,
                    ),
                    RibbonLargeButton(
                      key: const Key('view_web_layout'),
                      icon: Icons.web,
                      label: 'Web\nLayout',
                      tooltip: 'Continuous web-style layout',
                      onPressed: controller.setWebLayout,
                    ),
                  ],
                ),
              ),
              RibbonGroup(
                label: 'Show',
                child: Row(
                  children: [
                    Padding(
                      padding: const EdgeInsets.symmetric(horizontal: 4),
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          _RibbonCheckbox(
                            key: const Key('view_show_ruler'),
                            label: 'Ruler',
                            value: controller.showRuler,
                            onChanged: (_) => controller.toggleRuler(),
                          ),
                          const SizedBox(height: 4),
                          _RibbonCheckbox(
                            key: const Key('view_show_navigation_pane'),
                            label: 'Navigation Pane',
                            value: controller.showNavigationPane,
                            onChanged: (_) => controller.toggleNavigationPane(),
                          ),
                        ],
                      ),
                    ),
                    RibbonLargeButton(
                      icon: Icons.format_list_bulleted,
                      label: 'Document\nOutline',
                      tooltip: 'Show navigation pane outline (headings)',
                      onPressed: controller.showNavigationOutline,
                    ),
                    Builder(
                      builder: (context) => RibbonLargeButton(
                        key: const Key('goto_button'),
                        icon: Icons.arrow_forward,
                        label: 'Go To',
                        tooltip: 'Go to page, bookmark, or heading',
                        onPressed: () => controller.openGoToDialog(context),
                      ),
                    ),
                  ],
                ),
              ),
              RibbonGroup(
                label: 'Zoom',
                child: Row(
                  children: [
                    Builder(
                      builder: (context) => RibbonLargeButton(
                        key: const Key('view_zoom'),
                        icon: Icons.zoom_in,
                        label: 'Zoom',
                        tooltip: 'Choose zoom level',
                        onPressed: () => controller.openZoomDialog(context),
                      ),
                    ),
                    RibbonLargeButton(
                      key: const Key('view_one_page'),
                      icon: Icons.fit_screen,
                      label: 'One\nPage',
                      tooltip: 'Zoom to fit one page',
                      onPressed: controller.zoomToOnePage,
                    ),
                    RibbonLargeButton(
                      key: const Key('view_multiple_pages'),
                      icon: Icons.view_week,
                      label: 'Multiple\nPages',
                      tooltip: 'Zoom to fit two pages side by side',
                      onPressed: controller.zoomToMultiplePages,
                    ),
                  ],
                ),
              ),
              RibbonGroup(
                label: 'Window',
                child: Row(
                  children: [
                    Builder(
                      builder: (context) => RibbonLargeButton(
                        key: const Key('view_new_window'),
                        icon: Icons.view_sidebar,
                        label: 'New\nWindow',
                        tooltip: 'Open another window on this document',
                        onPressed: () => _openNewWindow(context),
                      ),
                    ),
                    RibbonLargeButton(
                      key: const Key('view_arrange_all'),
                      icon: Icons.view_array,
                      label: 'Arrange\nAll',
                      tooltip: 'Tile document views in a split layout',
                      onPressed: controller.arrangeAllViews,
                    ),
                    RibbonLargeButton(
                      key: const Key('view_split'),
                      icon: Icons.vertical_split,
                      label: 'Split',
                      tooltip: controller.splitView
                          ? 'Close split view'
                          : 'Split the window',
                      onPressed: controller.toggleSplitView,
                    ),
                  ],
                ),
              ),
              RibbonGroup(
                label: 'Help',
                showDivider: false,
                child: Row(
                  children: [
                    Builder(
                      builder: (context) => RibbonLargeButton(
                        key: const Key('view_about'),
                        icon: Icons.info_outline,
                        label: 'About\nTutuaword',
                        tooltip: 'Help, support, and feedback',
                        onPressed: () => controller.openAboutDialog(context),
                      ),
                    ),
                    Builder(
                      builder: (context) => RibbonLargeButton(
                        key: const Key('view_settings'),
                        icon: Icons.settings_outlined,
                        label: 'App\nSettings',
                        tooltip: 'Ribbon and tab colors',
                        onPressed: () => controller.openSettingsDialog(context),
                      ),
                    ),
                  ],
                ),
              ),
          ],
        );
      },
    );
  }
}

class _RibbonCheckbox extends StatelessWidget {
  const _RibbonCheckbox({
    super.key,
    required this.label,
    required this.value,
    required this.onChanged,
  });

  final String label;
  final bool value;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: () => onChanged(!value),
      borderRadius: BorderRadius.circular(3),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 1),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            SizedBox(
              width: 18,
              height: 18,
              child: Transform.scale(
                scale: 0.78,
                child: Checkbox(
                  value: value,
                  onChanged: (v) => onChanged(v ?? false),
                  materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                  visualDensity: VisualDensity.compact,
                  splashRadius: 0,
                ),
              ),
            ),
            const SizedBox(width: 6),
            Text(
              label,
              style: WordTheme.ribbonLabel.copyWith(fontSize: 11, height: 1.1),
            ),
          ],
        ),
      ),
    );
  }
}
