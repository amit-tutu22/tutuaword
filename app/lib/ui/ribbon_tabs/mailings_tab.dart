import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

class MailingsTab extends StatelessWidget {
  const MailingsTab({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        return RibbonTabScroller(
          children: [
            RibbonGroup(
              label: 'Start Mail Merge',
              child: Row(
                children: [
                  Builder(
                    builder: (context) => RibbonLargeButton(
                      key: const Key('start_mail_merge'),
                      icon: Icons.merge_type,
                      label: 'Start Mail\nMerge',
                      tooltip: 'Load CSV data source',
                      onPressed: () => controller.startMailMerge(context),
                    ),
                  ),
                  RibbonLargeButton(
                    key: const Key('finish_mail_merge'),
                    icon: Icons.done_all,
                    label: 'Finish &\nMerge',
                    tooltip: 'Apply current row to document',
                    onPressed: controller.hasMailMergeData
                        ? () => controller.finishMailMerge()
                        : null,
                  ),
                ],
              ),
            ),
            RibbonGroup(
              label: 'Create',
              child: Row(
                children: [
                  Builder(
                    builder: (context) => RibbonLargeButton(
                      key: const Key('create_envelope'),
                      icon: Icons.mail_outline,
                      label: 'Envelopes',
                      tooltip: 'Create envelope document',
                      onPressed: () => controller.createEnvelope(context),
                    ),
                  ),
                  Builder(
                    builder: (context) => RibbonLargeButton(
                      key: const Key('create_label'),
                      icon: Icons.label_outline,
                      label: 'Labels',
                      tooltip: 'Create label document',
                      onPressed: () => controller.createLabel(context),
                    ),
                  ),
                ],
              ),
            ),
            RibbonGroup(
              label: 'Rules',
              child: Row(
                children: [
                  RibbonLargeButton(
                    key: const Key('insert_next_record'),
                    icon: Icons.skip_next_outlined,
                    label: 'Next\nRecord',
                    tooltip: 'Insert NEXT mail-merge field',
                    onPressed: controller.hasMailMergeData
                        ? () => controller.insertNextRecordField(context)
                        : null,
                  ),
                  Builder(
                    builder: (context) => RibbonLargeButton(
                      key: const Key('insert_merge_if'),
                      icon: Icons.rule_folder_outlined,
                      label: 'Rules',
                      tooltip: 'Insert IF mail-merge field',
                      onPressed: controller.hasMailMergeData
                          ? () => controller.insertMergeIfField(context)
                          : null,
                    ),
                  ),
                ],
              ),
            ),
            RibbonGroup(
              label: 'Write & Insert Fields',
              showDivider: false,
              child: Builder(
                builder: (context) => RibbonLargeButton(
                  key: const Key('insert_merge_field'),
                  icon: Icons.edit_note,
                  label: 'Insert\nMerge Field',
                  tooltip: 'Insert MERGEFIELD',
                  onPressed: () => controller.insertMergeField(context),
                ),
              ),
            ),
          ],
        );
      },
    );
  }
}
