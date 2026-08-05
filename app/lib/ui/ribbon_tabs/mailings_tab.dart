import 'package:flutter/material.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

class MailingsTab extends StatelessWidget {
  const MailingsTab({super.key});

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          RibbonGroup(
            label: 'Create',
            child: Row(
              children: [
                RibbonLargeButton(icon: Icons.mail_outline, label: 'Envelopes', onPressed: null),
                RibbonLargeButton(icon: Icons.label_outline, label: 'Labels', onPressed: null),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Start Mail Merge',
            child: RibbonLargeButton(icon: Icons.merge_type, label: 'Start Mail\nMerge', onPressed: null),
          ),
          RibbonGroup(
            label: 'Write & Insert Fields',
            showDivider: false,
            child: Row(
              children: [
                RibbonLargeButton(icon: Icons.edit_note, label: 'Write &\nInsert Fields', onPressed: null),
                RibbonLargeButton(icon: Icons.rule, label: 'Rules', onPressed: null),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
