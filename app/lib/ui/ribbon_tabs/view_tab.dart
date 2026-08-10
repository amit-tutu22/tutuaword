import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';
import 'package:tutuaword/ui/word_theme.dart';

class ViewTab extends StatelessWidget {
  const ViewTab({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        return RibbonTabScroller(
          children: [
              RibbonGroup(
                label: 'Views',
                child: Row(
                  children: [
                    RibbonLargeButton(icon: Icons.article_outlined, label: 'Read\nMode', onPressed: null),
                    RibbonLargeButton(
                      icon: Icons.print_outlined,
                      label: 'Print\nLayout',
                      tooltip: 'Print layout editing view',
                      onPressed: controller.printPreview ? controller.togglePrintPreview : null,
                    ),
                    RibbonLargeButton(
                      icon: Icons.preview_outlined,
                      label: 'Print\nPreview',
                      tooltip: 'Read-only print preview',
                      onPressed: !controller.printPreview ? controller.togglePrintPreview : null,
                    ),
                    RibbonLargeButton(icon: Icons.web, label: 'Web\nLayout', onPressed: null),
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
                    RibbonLargeButton(icon: Icons.zoom_in, label: 'Zoom', onPressed: null),
                    RibbonLargeButton(icon: Icons.fit_screen, label: 'One\nPage', onPressed: null),
                    RibbonLargeButton(icon: Icons.view_week, label: 'Multiple\nPages', onPressed: null),
                  ],
                ),
              ),
              RibbonGroup(
                label: 'Window',
                showDivider: false,
                child: Row(
                  children: [
                    RibbonLargeButton(icon: Icons.view_sidebar, label: 'New\nWindow', onPressed: null),
                    RibbonLargeButton(icon: Icons.view_array, label: 'Arrange\nAll', onPressed: null),
                    RibbonLargeButton(icon: Icons.vertical_split, label: 'Split', onPressed: null),
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
