import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

const kTrackChangeAcceptTooltip = 'Accept the track-change revision at the caret';
const kTrackChangeRejectTooltip = 'Reject the track-change revision at the caret';
const kTrackChangeNextTooltip = 'Go to the next tracked change';
const kTrackChangePreviousTooltip = 'Go to the previous tracked change';

class ReviewTab extends StatelessWidget {
  const ReviewTab({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    return RibbonTabScroller(
      children: [
          RibbonGroup(
            label: 'Proofing',
            child: Row(
              children: [
                RibbonLargeButton(
                  key: const Key('spell_check'),
                  icon: Icons.spellcheck,
                  label: 'Spelling &\nGrammar',
                  onPressed: () => controller.proofDocument(),
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
            child: RibbonLargeButton(
              key: const Key('insert_comment'),
              icon: Icons.comment_outlined,
              label: 'New\nComment',
              onPressed: () => controller.insertComment(context),
            ),
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
                RibbonIconButton(
                  key: const Key('accept_revision'),
                  icon: Icons.check,
                  label: 'Accept',
                  tooltip: kTrackChangeAcceptTooltip,
                  onPressed: controller.acceptRevisionAtCaret,
                ),
                RibbonIconButton(
                  key: const Key('reject_revision'),
                  icon: Icons.close,
                  label: 'Reject',
                  tooltip: kTrackChangeRejectTooltip,
                  onPressed: controller.rejectRevisionAtCaret,
                ),
                RibbonIconButton(
                  key: const Key('next_revision'),
                  icon: Icons.arrow_downward,
                  label: 'Next\nChange',
                  tooltip: kTrackChangeNextTooltip,
                  onPressed: controller.gotoNextRevision,
                ),
                RibbonIconButton(
                  key: const Key('previous_revision'),
                  icon: Icons.arrow_upward,
                  label: 'Previous\nChange',
                  tooltip: kTrackChangePreviousTooltip,
                  onPressed: controller.gotoPreviousRevision,
                ),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Export',
            child: RibbonLargeButton(
              icon: Icons.picture_as_pdf_outlined,
              label: 'Export\nPDF',
              tooltip: 'Export as PDF (basic layout; font embedding still in progress)',
              onPressed: () => controller.exportPdf(),
            ),
          ),
          RibbonGroup(
            label: 'Changes',
            showDivider: false,
            child: Row(
              children: [
                RibbonIconButton(
                  key: const Key('compare_documents'),
                  icon: Icons.compare,
                  label: 'Compare',
                  onPressed: () => controller.compareWithText(controller.documentText),
                ),
                RibbonIconButton(
                  key: const Key('restrict_editing'),
                  icon: Icons.lock_outline,
                  label: 'Restrict\nEditing',
                  onPressed: controller.toggleRestrictEditing,
                ),
              ],
            ),
          ),
      ],
    );
  }
}
