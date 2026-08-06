import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';
import 'package:tutuaword/ui/word_theme.dart';

class ViewTab extends StatelessWidget {
  const ViewTab({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          RibbonGroup(
            label: 'Views',
            child: Row(
              children: [
                RibbonLargeButton(icon: Icons.article_outlined, label: 'Read\nMode', onPressed: null),
                RibbonLargeButton(
                  icon: Icons.print_outlined,
                  label: 'Print\nLayout',
                  tooltip: controller.printPreview
                      ? 'Return to print layout'
                      : 'Print layout view',
                  onPressed: controller.togglePrintPreview,
                ),
                RibbonLargeButton(icon: Icons.web, label: 'Web\nLayout', onPressed: null),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Show',
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                _RibbonCheckbox(
                  label: 'Ruler',
                  value: controller.showRuler,
                  onChanged: (_) => controller.toggleRuler(),
                ),
                _RibbonCheckbox(
                  label: 'Navigation\nPane',
                  value: controller.showNavigationPane,
                  onChanged: (_) => controller.toggleNavigationPane(),
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
      ),
    );
  }
}

class _RibbonCheckbox extends StatelessWidget {
  const _RibbonCheckbox({
    required this.label,
    required this.value,
    required this.onChanged,
  });

  final String label;
  final bool value;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => onChanged(!value),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          SizedBox(
            width: 14,
            height: 14,
            child: Checkbox(
              value: value,
              onChanged: (v) => onChanged(v ?? false),
              materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
              visualDensity: VisualDensity.compact,
            ),
          ),
          const SizedBox(width: 4),
          Text(label, style: WordTheme.ribbonLabel.copyWith(fontSize: 10)),
        ],
      ),
    );
  }
}
