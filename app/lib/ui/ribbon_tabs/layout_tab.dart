import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/page_setup.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';
import 'package:tutuaword/ui/word_theme.dart';

class LayoutTab extends StatelessWidget {
  const LayoutTab({super.key, required this.controller});

  final EditorController controller;

  Future<void> _showPresetMenu(
    BuildContext context,
    RenderBox anchor,
    List<String> items,
    ValueChanged<String> onSelected,
  ) async {
    final overlay = Overlay.of(context).context.findRenderObject() as RenderBox;
    final origin = anchor.localToGlobal(Offset.zero, ancestor: overlay);
    final selected = await showMenu<String>(
      context: context,
      position: RelativeRect.fromLTRB(
        origin.dx,
        origin.dy + anchor.size.height,
        origin.dx + anchor.size.width,
        origin.dy + anchor.size.height + 4,
      ),
      constraints: const BoxConstraints(minWidth: 120, maxWidth: 200, maxHeight: 280),
      items: items
          .map(
            (item) => PopupMenuItem<String>(
              value: item,
              height: 28,
              child: Text(item, style: WordTheme.ribbonLabel),
            ),
          )
          .toList(),
    );
    if (selected != null) onSelected(selected);
  }

  Future<void> _openMarginsMenu(BuildContext context) async {
    final box = context.findRenderObject() as RenderBox?;
    if (box == null) return;
    await _showPresetMenu(
      context,
      box,
      PageSetupPresets.marginNames,
      controller.applyMarginPreset,
    );
  }

  Future<void> _openOrientationMenu(BuildContext context) async {
    final box = context.findRenderObject() as RenderBox?;
    if (box == null) return;
    await _showPresetMenu(
      context,
      box,
      const ['Portrait', 'Landscape'],
      (value) => controller.setOrientation(landscape: value == 'Landscape'),
    );
  }

  Future<void> _openSizeMenu(BuildContext context) async {
    final box = context.findRenderObject() as RenderBox?;
    if (box == null) return;
    await _showPresetMenu(
      context,
      box,
      PageSetupPresets.pageSizeNames,
      controller.applyPageSizePreset,
    );
  }

  Future<void> _openColumnsMenu(BuildContext context) async {
    final box = context.findRenderObject() as RenderBox?;
    if (box == null) return;
    await _showPresetMenu(
      context,
      box,
      PageSetupPresets.columnCountNames,
      (value) => controller.applyColumnCount(
        switch (value) {
          'Two' => 2,
          'Three' => 3,
          _ => 1,
        },
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        return RibbonTabScroller(
          children: [
            RibbonGroup(
              label: 'Page Setup',
              child: Row(
                children: [
                  Builder(
                    builder: (context) => RibbonLargeButton(
                      icon: Icons.description_outlined,
                      label: 'Margins',
                      tooltip: controller.marginPresetName,
                      onPressed: () => _openMarginsMenu(context),
                    ),
                  ),
                  Builder(
                    builder: (context) => RibbonLargeButton(
                      icon: controller.isLandscape
                          ? Icons.crop_landscape
                          : Icons.crop_portrait,
                      label: 'Orientation',
                      tooltip: controller.isLandscape ? 'Landscape' : 'Portrait',
                      onPressed: () => _openOrientationMenu(context),
                    ),
                  ),
                  Builder(
                    builder: (context) => RibbonLargeButton(
                      icon: Icons.aspect_ratio,
                      label: 'Size',
                      tooltip: controller.pageSizePresetName,
                      onPressed: () => _openSizeMenu(context),
                    ),
                  ),
                  Builder(
                    builder: (context) => RibbonLargeButton(
                      icon: Icons.view_column_outlined,
                      label: 'Columns',
                      tooltip: controller.columnCountLabel,
                      onPressed: () => _openColumnsMenu(context),
                    ),
                  ),
                ],
              ),
            ),
            RibbonGroup(
              label: 'Breaks',
              child: Row(
                children: [
                  RibbonLargeButton(
                    icon: Icons.insert_page_break,
                    label: 'Page',
                    tooltip: 'Page Break',
                    onPressed: controller.insertPageBreak,
                  ),
                  RibbonLargeButton(
                    icon: Icons.view_day_outlined,
                    label: 'Section',
                    tooltip: 'Section Break (Next Page)',
                    onPressed: controller.insertSectionBreak,
                  ),
                ],
              ),
            ),
            RibbonGroup(
              label: 'Paragraph',
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Row(
                    children: [
                      RibbonIconButton(
                        icon: Icons.format_indent_decrease,
                        tooltip: 'Decrease Indent',
                        onPressed: controller.decreaseIndent,
                      ),
                      RibbonIconButton(
                        icon: Icons.format_indent_increase,
                        tooltip: 'Increase Indent',
                        onPressed: controller.increaseIndent,
                      ),
                      RibbonIconButton(
                        icon: Icons.keyboard_tab,
                        label: 'Tabs',
                        tooltip: 'Tab Stops',
                        onPressed: () => controller.showTabStopsDialog(context),
                      ),
                    ],
                  ),
                  Row(
                    children: [
                      RibbonIconButton(
                        icon: Icons.space_bar,
                        label: 'Before',
                        tooltip: 'Increase space before paragraph',
                        onPressed: controller.increaseSpaceBefore,
                      ),
                      RibbonIconButton(
                        icon: Icons.vertical_align_bottom,
                        label: 'After',
                        tooltip: 'Increase space after paragraph',
                        onPressed: controller.increaseSpaceAfter,
                      ),
                    ],
                  ),
                ],
              ),
            ),
            RibbonGroup(
              label: 'Table',
              child: Row(
                children: [
                  RibbonLargeButton(
                    icon: Icons.table_rows,
                    label: 'Delete\nRow',
                    onPressed: controller.deleteTableRow,
                  ),
                  RibbonLargeButton(
                    icon: Icons.view_column,
                    label: 'Delete\nColumn',
                    onPressed: controller.deleteTableColumn,
                  ),
                  RibbonLargeButton(
                    icon: Icons.merge_type,
                    label: 'Merge\nCells',
                    onPressed: controller.mergeTableCells,
                  ),
                  RibbonLargeButton(
                    icon: Icons.call_split,
                    label: 'Split\nCell',
                    onPressed: controller.splitTableCell,
                  ),
                  RibbonLargeButton(
                    key: const Key('table_design_button'),
                    icon: Icons.border_all,
                    label: 'Table\nDesign',
                    onPressed: () => controller.showTableDesignDialog(context),
                  ),
                ],
              ),
            ),
            RibbonGroup(
              label: 'Table Data',
              child: Row(
                children: [
                  RibbonLargeButton(
                    key: const Key('table_sort_asc'),
                    icon: Icons.arrow_upward,
                    label: 'Sort\nA→Z',
                    onPressed: controller.sortTableAscending,
                  ),
                  RibbonLargeButton(
                    icon: Icons.arrow_downward,
                    label: 'Sort\nZ→A',
                    onPressed: controller.sortTableDescending,
                  ),
                  RibbonLargeButton(
                    key: const Key('table_sum_formula'),
                    icon: Icons.functions,
                    label: 'Sum\nFormula',
                    onPressed: controller.insertTableSumFormula,
                  ),
                  RibbonLargeButton(
                    key: const Key('table_insert_nested'),
                    icon: Icons.grid_view,
                    label: 'Nested\nTable',
                    onPressed: controller.insertNestedTable,
                  ),
                ],
              ),
            ),
            RibbonGroup(
              label: 'Wrap Text',
              child: Row(
                children: [
                  RibbonLargeButton(
                    key: const Key('wrap_inline'),
                    icon: Icons.notes,
                    label: 'Inline',
                    tooltip: 'Inline with text',
                    onPressed: controller.hasSelectedImage
                        ? () => controller.setSelectedImageWrap(0)
                        : null,
                  ),
                  RibbonLargeButton(
                    key: const Key('wrap_square'),
                    icon: Icons.crop_square,
                    label: 'Square',
                    tooltip: 'Square text wrap',
                    onPressed: controller.hasSelectedImage
                        ? () => controller.setSelectedImageWrap(1)
                        : null,
                  ),
                  RibbonLargeButton(
                    key: const Key('wrap_behind'),
                    icon: Icons.layers,
                    label: 'Behind',
                    tooltip: 'Behind text',
                    onPressed: controller.hasSelectedImage
                        ? () => controller.setSelectedImageWrap(3)
                        : null,
                  ),
                ],
              ),
            ),
            RibbonGroup(
              label: 'Page Background',
              showDivider: false,
              child: Row(
                children: [
                  RibbonLargeButton(
                    icon: Icons.format_list_numbered,
                    label: 'Line\nNumbers',
                    tooltip: controller.lineNumbersEnabled ? 'On' : 'Off',
                    onPressed: () => controller.setLineNumbersEnabled(
                      !controller.lineNumbersEnabled,
                    ),
                  ),
                  Builder(
                    builder: (context) => RibbonLargeButton(
                      key: const Key('layout_page_borders'),
                      icon: Icons.border_style,
                      label: 'Page\nBorders',
                      tooltip: controller.hasPageBorders
                          ? 'Page borders on'
                          : 'Page borders',
                      onPressed: () => controller.editPageBorders(context),
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
