import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

class InsertTab extends StatelessWidget {
  const InsertTab({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          RibbonGroup(
            label: 'Pages',
            child: Row(
              children: [
                RibbonLargeButton(icon: Icons.insert_page_break, label: 'Page\nBreak', onPressed: null),
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
                  icon: Icons.image_outlined,
                  label: 'Pictures',
                  onPressed: controller.insertImage,
                ),
                RibbonLargeButton(icon: Icons.shape_line_outlined, label: 'Shapes', onPressed: null),
                RibbonLargeButton(icon: Icons.smart_display_outlined, label: 'Online\nVideo', onPressed: null),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Header & Footer',
            child: Row(
              children: [
                RibbonLargeButton(icon: Icons.vertical_align_top, label: 'Header', onPressed: null),
                RibbonLargeButton(icon: Icons.vertical_align_bottom, label: 'Footer', onPressed: null),
                RibbonLargeButton(icon: Icons.numbers, label: 'Page\nNumber', onPressed: null),
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
      ),
    );
  }
}
