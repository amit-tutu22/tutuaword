import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/ai_client.dart';

/// Audience-targeted rewrite tone picker (smart differentiator).
class AudienceRewriteDialog extends StatelessWidget {
  const AudienceRewriteDialog({super.key});

  static Future<AiRewriteTone?> pick(BuildContext context) {
    return showDialog<AiRewriteTone>(
      context: context,
      builder: (context) => const AudienceRewriteDialog(),
    );
  }

  @override
  Widget build(BuildContext context) {
    return SimpleDialog(
      title: const Text('Rewrite for audience'),
      children: [
        for (final entry in _audienceTones)
          SimpleDialogOption(
            key: Key('audience_tone_${entry.tone.name}'),
            onPressed: () => Navigator.of(context).pop(entry.tone),
            child: ListTile(
              title: Text(entry.label),
              subtitle: Text(entry.description),
            ),
          ),
      ],
    );
  }
}

class _AudienceTone {
  const _AudienceTone(this.tone, this.label, this.description);
  final AiRewriteTone tone;
  final String label;
  final String description;
}

const _audienceTones = [
  _AudienceTone(
    AiRewriteTone.executive,
    'Executive',
    'Concise, outcome-focused language',
  ),
  _AudienceTone(
    AiRewriteTone.student,
    'Student',
    'Clear explanations with simpler vocabulary',
  ),
  _AudienceTone(
    AiRewriteTone.customer,
    'Customer',
    'Friendly, benefit-oriented wording',
  ),
];
