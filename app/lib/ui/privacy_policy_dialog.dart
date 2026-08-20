import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/bridge/external_link.dart';
import 'package:tutuaword/bridge/support_links.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Full privacy policy text bundled with the app, with a link to the public copy.
class PrivacyPolicyDialog extends StatelessWidget {
  const PrivacyPolicyDialog({super.key, required this.body});

  final String body;

  static Future<void> show(BuildContext context) async {
    final body = await rootBundle.loadString('assets/legal/privacy_policy.md');
    if (!context.mounted) return;
    await showDialog<void>(
      context: context,
      barrierDismissible: true,
      builder: (context) => PrivacyPolicyDialog(body: body),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('privacy_policy_dialog'),
      title: const Text('Privacy Policy'),
      content: SizedBox(
        width: 520,
        height: 420,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              'Full policy: $kPrivacyPolicyUrl',
              style: const TextStyle(fontSize: 11, color: Color(0xFF666666)),
            ),
            const SizedBox(height: 8),
            Expanded(
              child: Scrollbar(
                child: SingleChildScrollView(
                  child: SelectableText(
                    body,
                    style: const TextStyle(
                      fontSize: 12,
                      height: 1.45,
                      color: WordTheme.ribbonText,
                    ),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('privacy_policy_open_online'),
          onPressed: () => openExternalUri(privacyPolicyUri),
          child: const Text('View online'),
        ),
        TextButton(
          key: const Key('privacy_policy_close'),
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
      ],
    );
  }
}
