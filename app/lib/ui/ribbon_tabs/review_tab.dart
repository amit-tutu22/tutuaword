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
                RibbonLargeButton(
                  key: const Key('check_accessibility'),
                  icon: Icons.accessibility_new,
                  label: 'Check\nAccessibility',
                  onPressed: () => controller.checkAccessibility(),
                ),
                RibbonLargeButton(
                  key: const Key('read_aloud'),
                  icon: controller.isReadingAloud
                      ? Icons.stop_circle_outlined
                      : Icons.record_voice_over_outlined,
                  label: controller.isReadingAloud ? 'Stop\nReading' : 'Read\nAloud',
                  tooltip: controller.isReadingAloud
                      ? 'Stop reading aloud'
                      : 'Read selection aloud',
                  onPressed: () => controller.toggleReadAloud(),
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('review_translate'),
                    icon: Icons.translate,
                    label: 'Translate',
                    tooltip: 'Translate selection with AI',
                    onPressed: () => controller.translateSelection(context),
                  ),
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('review_thesaurus'),
                    icon: Icons.book_outlined,
                    label: 'Thesaurus',
                    tooltip: 'Find synonyms for the selected word',
                    onPressed: () => controller.openThesaurus(context),
                  ),
                ),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Language',
            child: Builder(
              builder: (context) => RibbonLargeButton(
                key: const Key('review_language'),
                icon: Icons.language,
                label: 'Language',
                tooltip:
                    'Proofing language: ${controller.proofingLanguage.label}',
                onPressed: () => controller.chooseProofingLanguage(context),
              ),
            ),
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
            label: 'AI',
            child: Row(
              children: [
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('ai_rewrite'),
                    icon: Icons.auto_fix_high_outlined,
                    label: 'Rewrite',
                    tooltip: 'Rewrite selection with AI',
                    onPressed: () => controller.rewriteSelection(context),
                  ),
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('ai_chat'),
                    icon: Icons.chat_outlined,
                    label: 'Ask AI',
                    tooltip: 'Chat about this document',
                    onPressed: () => controller.openDocumentChat(context),
                  ),
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('ai_generate'),
                    icon: Icons.note_add_outlined,
                    label: 'Generate',
                    tooltip: 'Generate outline, minutes, or report',
                    onPressed: () => controller.openContentGenerate(context),
                  ),
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('ai_visual'),
                    icon: Icons.schema_outlined,
                    label: 'Visual',
                    tooltip: 'Suggest table, diagram, or timeline',
                    onPressed: () => controller.openVisualAssist(context),
                  ),
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('ai_smart_edit'),
                    icon: Icons.tips_and_updates_outlined,
                    label: 'Smart\nEdit',
                    tooltip: 'Suggest headings and TOC draft',
                    onPressed: () => controller.openSmartEdit(context),
                  ),
                ),
                Builder(
                  builder: (context) => RibbonLargeButton(
                    key: const Key('ai_settings'),
                    icon: Icons.auto_awesome_outlined,
                    label: 'AI\nSettings',
                    tooltip: 'AI routing mode and providers',
                    onPressed: () => controller.openAiSettings(context),
                  ),
                ),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Add-ins',
            child: Builder(
              builder: (context) => RibbonLargeButton(
                key: const Key('manage_plugins'),
                icon: Icons.extension_outlined,
                label: 'Plugins',
                tooltip: 'Manage WASM plugins',
                onPressed: () => controller.managePlugins(context),
              ),
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
