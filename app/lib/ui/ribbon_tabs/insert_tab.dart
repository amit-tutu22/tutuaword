import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

class InsertTab extends StatelessWidget {
  const InsertTab({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
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
                RibbonLargeButton(icon: Icons.view_agenda_outlined, label: 'Cover\nPage', onPressed: null),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Tables',
            child: RibbonLargeButton(
              icon: Icons.table_chart,
              label: 'Table',
              onPressed: controller.insertTable,
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
                  onPressed: controller.hasSelectedImage
                      ? () => controller.replaceSelectedImage()
                      : null,
                ),
                RibbonLargeButton(
                  key: const Key('rotate_picture'),
                  icon: Icons.rotate_right,
                  label: 'Rotate',
                  onPressed: controller.hasSelectedImage
                      ? () => controller.rotateSelectedImage()
                      : null,
                ),
                RibbonLargeButton(
                  key: const Key('insert_caption'),
                  icon: Icons.notes,
                  label: 'Caption',
                  onPressed: controller.hasSelectedImage
                      ? () => controller.insertSelectedImageCaption()
                      : null,
                ),
                RibbonLargeButton(
                  key: const Key('compress_picture'),
                  icon: Icons.compress,
                  label: 'Compress',
                  onPressed: controller.hasSelectedImage
                      ? () => controller.compressSelectedImage()
                      : null,
                ),
                RibbonLargeButton(
                  key: const Key('insert_smart_art'),
                  icon: Icons.account_tree_outlined,
                  label: 'SmartArt',
                  onPressed: () => controller.insertSmartArt(),
                ),
                RibbonLargeButton(
                  key: const Key('insert_chart'),
                  icon: Icons.bar_chart_outlined,
                  label: 'Chart',
                  onPressed: () => controller.insertChart(),
                ),
                RibbonLargeButton(
                  key: const Key('insert_text_box'),
                  icon: Icons.text_fields_outlined,
                  label: 'Text Box',
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
                RibbonLargeButton(
                  key: const Key('insert_shapes'),
                  icon: Icons.shape_line_outlined,
                  label: 'Shapes',
                  onPressed: () async {
                    final selected = await showMenu<int>(
                      context: context,
                      position: const RelativeRect.fromLTRB(200, 120, 200, 0),
                      items: const [
                        PopupMenuItem(
                          value: EditorController.shapeRectangle,
                          child: Text('Rectangle'),
                        ),
                        PopupMenuItem(
                          value: EditorController.shapeLine,
                          child: Text('Line'),
                        ),
                        PopupMenuItem(
                          value: EditorController.shapeEllipse,
                          child: Text('Ellipse'),
                        ),
                      ],
                    );
                    if (selected != null) {
                      await controller.insertShape(selected);
                    }
                  },
                ),
                RibbonLargeButton(icon: Icons.smart_display_outlined, label: 'Online\nVideo', onPressed: null),
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
            label: 'Text',
            showDivider: false,
            child: Row(
              children: [
                RibbonLargeButton(icon: Icons.text_fields, label: 'Text\nBox', onPressed: null),
                RibbonLargeButton(icon: Icons.functions, label: 'Symbol', onPressed: null),
              ],
            ),
          ),
      ],
    );
  }
}
