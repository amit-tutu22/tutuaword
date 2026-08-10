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
              label: 'Create',
              child: Row(
                children: [
                  RibbonLargeButton(
                    icon: Icons.mail_outline,
                    label: 'Envelopes',
                    onPressed: null,
                  ),
                  RibbonLargeButton(
                    icon: Icons.label_outline,
                    label: 'Labels',
                    onPressed: null,
                  ),
                ],
              ),
            ),
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
              label: 'Write & Insert Fields',
              showDivider: false,
              child: Row(
                children: [
                  Builder(
                    builder: (context) => RibbonLargeButton(
                      key: const Key('insert_merge_field'),
                      icon: Icons.edit_note,
                      label: 'Insert\nMerge Field',
                      tooltip: 'Insert MERGEFIELD',
                      onPressed: () => controller.insertMergeField(context),
                    ),
                  ),
                  RibbonLargeButton(
                    icon: Icons.rule,
                    label: 'Rules',
                    onPressed: null,
                  ),
                ],
              ),
            ),
          ],
        );
      },
    );
  }
}
