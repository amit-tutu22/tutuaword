import 'package:flutter/material.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

class DesignTab extends StatelessWidget {
  const DesignTab({super.key});

  @override
  Widget build(BuildContext context) {
    return RibbonTabScroller(
      children: [
          RibbonGroup(
            label: 'Document Formatting',
            child: Row(
              children: [
                StyleGalleryCard(label: 'Office', onPressed: null),
                StyleGalleryCard(label: 'Facet', onPressed: null),
                StyleGalleryCard(label: 'Ion', onPressed: null),
                RibbonIconButton(icon: Icons.chevron_right, onPressed: null),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Page Background',
            showDivider: false,
            child: Row(
              children: [
                RibbonLargeButton(icon: Icons.water_drop_outlined, label: 'Watermark', onPressed: null),
                RibbonLargeButton(icon: Icons.palette_outlined, label: 'Page\nColor', onPressed: null),
                RibbonLargeButton(icon: Icons.border_style, label: 'Page\nBorders', onPressed: null),
              ],
            ),
          ),
      ],
    );
  }
}
