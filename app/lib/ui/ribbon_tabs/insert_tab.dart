import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/illustration_pickers.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';
import 'package:tutuaword/ui/shape_picker.dart';
import 'package:tutuaword/ui/symbol_dialog.dart';
import 'package:tutuaword/ui/table_size_picker.dart';

class InsertTab extends StatelessWidget {
  const InsertTab({super.key, required this.controller});

  final EditorController controller;

  Future<void> _insertTable(BuildContext context) async {
    final size = await TableSizePicker.show(context);
    if (size == null) return;
    await controller.insertTable(rows: size.rows, cols: size.cols);
  }

  Future<void> _insertSmartArt(BuildContext context) async {
    final selected = await SmartArtPicker.show(context);
    if (selected == null) return;
    final type = switch (selected) {
      'hierarchy' => EditorController.smartArtHierarchy,
      'cycle' => EditorController.smartArtCycle,
      _ => EditorController.smartArtProcess,
    };
    await controller.insertSmartArt(diagramType: type);
  }

  Future<void> _insertChart(BuildContext context) async {
    final selected = await ChartPicker.show(context);
    if (selected == null) return;
    await controller.insertChart(chartType: selected);
    if (!context.mounted) return;
    await controller.editChartData(context);
  }

  Future<void> _insertShape(BuildContext context) async {
    final selected = await ShapePicker.show(context);
    if (selected == null) return;
    await controller.insertShape(selected);
  }

  Future<void> _insertEquation(BuildContext context) async {
    await controller.insertEquation(context);
  }

  Future<void> _insertSymbol(BuildContext context) async {
    await controller.insertSymbol(context);
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        final hasImage = controller.hasSelectedImage;
        const selectPictureTip = 'Select a picture in the document first';
        return RibbonTabScroller(
          children: [
          RibbonGroup(
            label: 'Pages',
            child: Row(
              children: [
                RibbonLargeButton(
                  icon: Icons.insert_page_break,
                  label: 'Page\nBreak',
                  onPressed: controller.insertPageBreak,
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('insert_cover_page'),
                    icon: Icons.view_agenda_outlined,
                    label: 'Cover\nPage',
                    tooltip: 'Insert a title cover page',
                    onPressed: () => controller.insertCoverPage(context),
                  ),
                ),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Tables',
            child: Builder(
              builder: (context) => RibbonLargeButton(
                key: const Key('insert_table_button'),
                icon: Icons.table_chart,
                label: 'Table',
                tooltip: 'Insert Table',
                onPressed: () => _insertTable(context),
                onDropdown: () => _insertTable(context),
              ),
            ),
          ),
          RibbonGroup(
            label: 'Illustrations',
            child: Row(
              children: [
                RibbonLargeButton(
                  key: const Key('insert_picture'),
                  icon: Icons.image_outlined,
                  label: 'Pictures',
                  onPressed: () => controller.insertImage(),
                ),
                RibbonLargeButton(
                  key: const Key('change_picture'),
                  icon: Icons.swap_horiz,
                  label: 'Change\nPicture',
                  tooltip: hasImage ? 'Replace the selected picture' : selectPictureTip,
                  onPressed: hasImage
                      ? () => controller.replaceSelectedImage()
                      : null,
                ),
                RibbonLargeButton(
                  key: const Key('rotate_picture'),
                  icon: Icons.rotate_right,
                  label: 'Rotate',
                  tooltip: hasImage ? 'Rotate the selected picture 90°' : selectPictureTip,
                  onPressed: hasImage
                      ? () => controller.rotateSelectedImage()
                      : null,
                ),
                RibbonLargeButton(
                  key: const Key('insert_caption'),
                  icon: Icons.notes,
                  label: 'Caption',
                  tooltip: hasImage ? 'Insert a caption under the picture' : selectPictureTip,
                  onPressed: hasImage
                      ? () => controller.insertSelectedImageCaption()
                      : null,
                ),
                RibbonLargeButton(
                  key: const Key('compress_picture'),
                  icon: Icons.compress,
                  label: 'Compress',
                  tooltip: hasImage ? 'Compress the selected picture' : selectPictureTip,
                  onPressed: hasImage
                      ? () => controller.compressSelectedImage()
                      : null,
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('insert_smart_art'),
                    icon: Icons.account_tree_outlined,
                    label: 'Smart\nArt',
                    tooltip: 'Insert SmartArt',
                    onPressed: () => _insertSmartArt(context),
                    onDropdown: () => _insertSmartArt(context),
                  ),
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('insert_chart'),
                    icon: Icons.bar_chart_outlined,
                    label: 'Chart',
                    tooltip: 'Insert Chart',
                    onPressed: () => _insertChart(context),
                    onDropdown: () => _insertChart(context),
                  ),
                ),
                RibbonLargeButton(
                  key: const Key('insert_text_box'),
                  icon: Icons.text_fields_outlined,
                  label: 'Text\nBox',
                  onPressed: () => controller.insertTextBox(),
                ),
                RibbonLargeButton(
                  key: const Key('insert_word_art'),
                  icon: Icons.font_download_outlined,
                  label: 'WordArt',
                  onPressed: () async {
                    final text = await showDialog<String>(
                      context: context,
                      builder: (ctx) {
                        final controller = TextEditingController(text: 'WordArt');
                        return AlertDialog(
                          title: const Text('Insert WordArt'),
                          content: TextField(
                            controller: controller,
                            autofocus: true,
                            decoration: const InputDecoration(labelText: 'Text'),
                            onSubmitted: (value) => Navigator.pop(ctx, value),
                          ),
                          actions: [
                            TextButton(
                              onPressed: () => Navigator.pop(ctx),
                              child: const Text('Cancel'),
                            ),
                            TextButton(
                              onPressed: () => Navigator.pop(ctx, controller.text),
                              child: const Text('Insert'),
                            ),
                          ],
                        );
                      },
                    );
                    if (text != null && text.isNotEmpty) {
                      await controller.insertWordArt(text);
                    }
                  },
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('insert_shapes'),
                    icon: Icons.shape_line_outlined,
                    label: 'Shapes',
                    tooltip: 'Insert Shape',
                    onPressed: () => _insertShape(context),
                    onDropdown: () => _insertShape(context),
                  ),
                ),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Header & Footer',
            child: Row(
              children: [
                RibbonLargeButton(
                  icon: Icons.vertical_align_top,
                  label: 'Header',
                  onPressed: controller.openHeaderEdit,
                ),
                RibbonLargeButton(
                  icon: Icons.vertical_align_bottom,
                  label: 'Footer',
                  onPressed: controller.openFooterEdit,
                ),
                RibbonLargeButton(
                  icon: Icons.numbers,
                  label: 'Page\nNumber',
                  onPressed: controller.insertPageNumberField,
                ),
                RibbonToggleButton(
                  icon: Icons.looks_one_outlined,
                  label: 'First\nPage',
                  selected: controller.differentFirstPage,
                  onPressed: () => controller.setDifferentFirstPage(!controller.differentFirstPage),
                ),
                RibbonToggleButton(
                  icon: Icons.view_week_outlined,
                  label: 'Odd &\nEven',
                  selected: controller.evenAndOddHeaders,
                  onPressed: () =>
                      controller.setEvenAndOddHeaders(!controller.evenAndOddHeaders),
                ),
                if (controller.canLinkHeaderFooter)
                  RibbonToggleButton(
                    icon: Icons.link,
                    label: 'Link to\nPrevious',
                    selected: controller.headerFooterLinked,
                    onPressed: () => controller.setHeaderFooterLinked(
                      !controller.headerFooterLinked,
                    ),
                  ),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Links',
            child: Row(
              children: [
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('insert_hyperlink'),
                    icon: Icons.link,
                    label: 'Link',
                    tooltip: 'Insert Hyperlink',
                    onPressed: () => controller.insertHyperlink(context),
                  ),
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('insert_bookmark_insert_tab'),
                    icon: Icons.bookmark_border,
                    label: 'Bookmark',
                    tooltip: 'Insert Bookmark',
                    onPressed: () => controller.insertBookmark(context),
                  ),
                ),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Text',
            showDivider: false,
            child: Row(
              children: [
                RibbonLargeButton(icon: Icons.text_fields, label: 'Text\nBox', onPressed: null),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('insert_form_field'),
                    icon: Icons.check_box_outlined,
                    label: 'Form\nField',
                    tooltip: 'Insert Form Field',
                    onPressed: () => controller.insertFormField(context),
                  ),
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('insert_equation'),
                    icon: Icons.functions,
                    label: 'Equation',
                    tooltip: 'Insert Equation',
                    onPressed: () => _insertEquation(context),
                  ),
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('insert_symbol'),
                    icon: Icons.abc,
                    label: 'Symbol',
                    tooltip: 'Insert Symbol',
                    onPressed: () => _insertSymbol(context),
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
