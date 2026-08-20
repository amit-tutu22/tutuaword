import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/bridge/app_version.dart';
import 'package:tutuaword/bridge/external_link.dart';
import 'package:tutuaword/bridge/support_links.dart';
import 'package:tutuaword/ui/keyboard_help_dialog.dart';
import 'package:tutuaword/ui/privacy_policy_dialog.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// About, help, support, and store feedback (title-bar gear / Help menu).
class TutuawordAboutDialog extends StatefulWidget {
  const TutuawordAboutDialog({super.key});

  static Future<void> show(BuildContext context) {
    return showDialog<void>(
      context: context,
      barrierDismissible: true,
      builder: (context) => const TutuawordAboutDialog(),
    );
  }

  @override
  State<TutuawordAboutDialog> createState() => _TutuawordAboutDialogState();
}

class _TutuawordAboutDialogState extends State<TutuawordAboutDialog> {
  Future<void> _open(Uri uri, {bool email = false}) async {
    final ok = email ? await openEmailUri(uri) : await openExternalUri(uri);
    if (!mounted || ok) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Could not open link on this device')),
    );
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('about_dialog'),
      title: const Text('About Tutuaword'),
      content: SizedBox(
        width: 420,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              ClipRRect(
                borderRadius: BorderRadius.circular(16),
                child: Image.asset(
                  'assets/icon/app_icon.png',
                  width: 96,
                  height: 96,
                  fit: BoxFit.cover,
                ),
              ),
              const SizedBox(height: 12),
              Text(
                'tutuaword',
                style: Theme.of(context).textTheme.titleLarge?.copyWith(
                      fontWeight: FontWeight.w600,
                    ),
              ),
              const SizedBox(height: 4),
              Text(
                appVersionLabel,
                key: const Key('about_version'),
                style: const TextStyle(fontSize: 12, color: Color(0xFF555555)),
              ),
              const SizedBox(height: 4),
              const Text(
                'Copyright © 2026 amit.blr76. All rights reserved.',
                textAlign: TextAlign.center,
                style: TextStyle(fontSize: 11, color: Color(0xFF666666)),
              ),
              const SizedBox(height: 16),
              const Align(
                alignment: Alignment.centerLeft,
                child: Text(
                  'AI-native document editor for DOCX and more.',
                  style: TextStyle(fontSize: 13, color: WordTheme.ribbonText),
                ),
              ),
              const SizedBox(height: 16),
              _LinkTile(
                key: const Key('about_keyboard_help'),
                icon: Icons.keyboard_alt_outlined,
                label: 'Keyboard shortcuts',
                subtitle: 'Word-compatible chord map (F1)',
                onTap: () {
                  final navigator = Navigator.of(context, rootNavigator: true);
                  navigator.pop();
                  KeyboardHelpDialog.show(navigator.context);
                },
              ),
              _LinkTile(
                key: const Key('about_contact_support'),
                icon: Icons.mail_outline,
                label: 'Contact support',
                subtitle: kSupportEmail,
                onTap: () => _open(mailtoSupportUri, email: true),
              ),
              _LinkTile(
                key: const Key('about_send_feedback'),
                icon: Icons.feedback_outlined,
                label: 'Send feedback',
                subtitle: kSupportEmail,
                onTap: () => _open(mailtoFeedbackUri, email: true),
              ),
              _LinkTile(
                key: const Key('about_privacy_policy'),
                icon: Icons.privacy_tip_outlined,
                label: 'Privacy Policy',
                subtitle: 'Local copy · also online',
                onTap: () {
                  final navigator = Navigator.of(context, rootNavigator: true);
                  navigator.pop();
                  PrivacyPolicyDialog.show(navigator.context);
                },
              ),
              if (showGooglePlayFeedback)
                _LinkTile(
                  key: const Key('about_play_store_feedback'),
                  icon: Icons.shop_outlined,
                  label: 'Rate on Google Play',
                  subtitle: 'Leave a review or feedback',
                  onTap: () => _open(googlePlayListingUri),
                ),
              if (showAppleStoreFeedback)
                _LinkTile(
                  key: const Key('about_app_store_feedback'),
                  icon: Icons.apple,
                  label: 'Rate on App Store',
                  subtitle: appleAppStoreReviewUri != null
                      ? 'Leave a review in the App Store'
                      : 'Open Tutuaword in the App Store',
                  onTap: () => _open(appleAppStoreFeedbackUri),
                ),
              const SizedBox(height: 8),
              Align(
                alignment: Alignment.centerLeft,
                child: TextButton.icon(
                  key: const Key('about_copy_email'),
                  onPressed: () async {
                    await Clipboard.setData(
                      const ClipboardData(text: kSupportEmail),
                    );
                    if (!context.mounted) return;
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(content: Text('Email copied')),
                    );
                  },
                  icon: const Icon(Icons.copy, size: 16),
                  label: const Text('Copy support email'),
                ),
              ),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          key: const Key('about_close'),
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
      ],
    );
  }
}

class _LinkTile extends StatelessWidget {
  const _LinkTile({
    super.key,
    required this.icon,
    required this.label,
    this.subtitle,
    this.onTap,
    this.enabled = true,
  });

  final IconData icon;
  final String label;
  final String? subtitle;
  final VoidCallback? onTap;
  final bool enabled;

  @override
  Widget build(BuildContext context) {
    final color = enabled ? WordTheme.ribbonText : const Color(0xFF999999);
    return ListTile(
      contentPadding: EdgeInsets.zero,
      leading: Icon(icon, size: 22, color: color),
      title: Text(label, style: TextStyle(fontSize: 13, color: color)),
      subtitle: subtitle == null
          ? null
          : Text(
              subtitle!,
              style: TextStyle(
                fontSize: 11,
                color: enabled ? const Color(0xFF666666) : const Color(0xFFAAAAAA),
              ),
            ),
      trailing: enabled ? const Icon(Icons.open_in_new, size: 16) : null,
      onTap: enabled ? onTap : null,
    );
  }
}
