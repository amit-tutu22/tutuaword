import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

class ReferencesTab extends StatelessWidget {
  const ReferencesTab({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    return RibbonTabScroller(
      children: [
        RibbonGroup(
          label: 'Table of Contents',
          child: RibbonLargeButton(
            key: const Key('insert_table_of_contents'),
            icon: Icons.list_alt,
            label: 'Table of\nContents',
            onPressed: () => controller.insertTableOfContents(context),
          ),
        ),
        RibbonGroup(
          label: 'Footnotes',
          child: Row(
            children: [
              RibbonLargeButton(
                key: const Key('insert_footnote'),
                icon: Icons.note_add_outlined,
                label: 'Insert\nFootnote',
                onPressed: () => controller.insertFootnote(context),
              ),
              RibbonLargeButton(
                icon: Icons.note_outlined,
                label: 'Insert\nEndnote',
                onPressed: null,
              ),
            ],
          ),
        ),
        RibbonGroup(
          label: 'Citations & Bibliography',
          child: Row(
            children: [
              RibbonLargeButton(
                key: const Key('insert_bibliography'),
                icon: Icons.menu_book_outlined,
                label: 'Bibliography',
                onPressed: () => controller.insertBibliography(context),
              ),
              RibbonLargeButton(
                key: const Key('insert_citation'),
                icon: Icons.format_quote,
                label: 'Insert\nCitation',
                onPressed: () => controller.insertCitation(context),
              ),
            ],
          ),
        ),
        RibbonGroup(
          label: 'Index & Cross-Reference',
          child: Row(
            children: [
              RibbonLargeButton(
                key: const Key('insert_bookmark'),
                icon: Icons.bookmark_outline,
                label: 'Insert\nBookmark',
                onPressed: () => controller.insertBookmark(context),
              ),
              RibbonLargeButton(
                key: const Key('insert_cross_reference'),
                icon: Icons.link,
                label: 'Cross-\nReference',
                onPressed: () => controller.insertCrossReference(context),
              ),
              RibbonLargeButton(
                key: const Key('insert_index'),
                icon: Icons.find_in_page_outlined,
                label: 'Insert\nIndex',
                onPressed: () => controller.insertIndex(context),
              ),
            ],
          ),
        ),
        RibbonGroup(
          label: 'Captions',
          showDivider: false,
          child: Row(
            children: [
              RibbonLargeButton(
                icon: Icons.label_outline,
                label: 'Insert\nCaption',
                onPressed: null,
              ),
              RibbonLargeButton(
                icon: Icons.list,
                label: 'Insert Table\nof Figures',
                onPressed: null,
              ),
            ],
          ),
        ),
      ],
    );
  }
}
