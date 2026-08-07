import 'package:flutter/material.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

class ReferencesTab extends StatelessWidget {
  const ReferencesTab({super.key});

  @override
  Widget build(BuildContext context) {
    return RibbonTabScroller(
      children: [
          RibbonGroup(
            label: 'Table of Contents',
            child: RibbonLargeButton(icon: Icons.list_alt, label: 'Table of\nContents', onPressed: null),
          ),
          RibbonGroup(
            label: 'Footnotes',
            child: Row(
              children: [
                RibbonLargeButton(icon: Icons.note_add_outlined, label: 'Insert\nFootnote', onPressed: null),
                RibbonLargeButton(icon: Icons.note_outlined, label: 'Insert\nEndnote', onPressed: null),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Citations & Bibliography',
            child: Row(
              children: [
                RibbonLargeButton(icon: Icons.menu_book_outlined, label: 'Bibliography', onPressed: null),
                RibbonLargeButton(icon: Icons.format_quote, label: 'Insert\nCitation', onPressed: null),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Captions',
            showDivider: false,
            child: Row(
              children: [
                RibbonLargeButton(icon: Icons.label_outline, label: 'Insert\nCaption', onPressed: null),
                RibbonLargeButton(icon: Icons.list, label: 'Insert Table\nof Figures', onPressed: null),
              ],
            ),
          ),
      ],
    );
  }
}
