import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Full privacy policy text bundled with the app.
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
      actions: [
        TextButton(
          key: const Key('privacy_policy_close'),
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
      ],
    );
  }
}
