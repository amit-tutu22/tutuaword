import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

class ReviewTab extends StatelessWidget {
  const ReviewTab({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          RibbonGroup(
            label: 'Proofing',
            child: Row(
              children: [
                RibbonLargeButton(
                  icon: Icons.spellcheck,
                  label: 'Spelling &\nGrammar',
                  onPressed: () => controller.spellCheckDocument(),
                ),
                RibbonLargeButton(icon: Icons.translate, label: 'Translate', onPressed: null),
                RibbonLargeButton(icon: Icons.book_outlined, label: 'Thesaurus', onPressed: null),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Language',
            child: RibbonLargeButton(icon: Icons.language, label: 'Language', onPressed: null),
          ),
          RibbonGroup(
            label: 'Comments',
            child: RibbonLargeButton(icon: Icons.comment_outlined, label: 'New\nComment', onPressed: null),
          ),
          RibbonGroup(
            label: 'Tracking',
            child: Row(
              children: [
                RibbonToggleButton(
                  icon: Icons.track_changes,
                  label: 'Track\nChanges',
                  selected: controller.trackChanges,
                  onPressed: controller.toggleTrackChanges,
                ),
                RibbonIconButton(icon: Icons.check, label: 'Accept', onPressed: null),
                RibbonIconButton(icon: Icons.close, label: 'Reject', onPressed: null),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Changes',
            showDivider: false,
            child: Row(
              children: [
                RibbonIconButton(icon: Icons.compare, label: 'Compare', onPressed: null),
                RibbonIconButton(icon: Icons.lock_outline, label: 'Restrict\nEditing', onPressed: null),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
