import 'package:flutter/material.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

class LayoutTab extends StatelessWidget {
  const LayoutTab({super.key});

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          RibbonGroup(
            label: 'Page Setup',
            child: Row(
              children: [
                RibbonLargeButton(icon: Icons.description_outlined, label: 'Margins', onPressed: null),
                RibbonLargeButton(icon: Icons.crop_portrait, label: 'Orientation', onPressed: null),
                RibbonLargeButton(icon: Icons.aspect_ratio, label: 'Size', onPressed: null),
                RibbonLargeButton(icon: Icons.view_column_outlined, label: 'Columns', onPressed: null),
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
                    RibbonIconButton(icon: Icons.format_indent_decrease, onPressed: null),
                    RibbonIconButton(icon: Icons.format_indent_increase, onPressed: null),
                  ],
                ),
                Row(
                  children: [
                    RibbonIconButton(icon: Icons.space_bar, label: 'Before', onPressed: null),
                    RibbonIconButton(icon: Icons.space_bar, label: 'After', onPressed: null),
                  ],
                ),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Page Background',
            showDivider: false,
            child: RibbonLargeButton(icon: Icons.border_style, label: 'Page\nBorders', onPressed: null),
          ),
        ],
      ),
    );
  }
}
