import 'package:flutter/material.dart';
import 'package:tutuaword/ui/app_chrome_theme.dart';
import 'package:tutuaword/ui/app_theme_controller.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// App appearance: ribbon / tab / title-bar accent color.
class AppSettingsDialog extends StatelessWidget {
  const AppSettingsDialog({super.key, this.theme});

  final AppThemeController? theme;

  static Future<void> show(BuildContext context, {AppThemeController? theme}) {
    return showDialog<void>(
      context: context,
      barrierDismissible: true,
      builder: (context) => AppSettingsDialog(theme: theme),
    );
  }

  @override
  Widget build(BuildContext context) {
    final controller = theme ?? AppThemeController.instance;
    return AlertDialog(
      key: const Key('app_settings_dialog'),
      title: const Text('Settings'),
      content: SizedBox(
        width: 420,
        child: ListenableBuilder(
          listenable: controller,
          builder: (context, _) {
            return Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Ribbon & tabs',
                  style: Theme.of(context).textTheme.titleSmall?.copyWith(
                        fontWeight: FontWeight.w600,
                      ),
                ),
                const SizedBox(height: 6),
                Text(
                  'Choose an accent for the title bar, tab strip, ribbon, and status bar.',
                  style: WordTheme.ribbonLabel.copyWith(
                    fontSize: 12,
                    color: const Color(0xFF555555),
                  ),
                ),
                const SizedBox(height: 14),
                Wrap(
                  spacing: 10,
                  runSpacing: 12,
                  children: [
                    for (final preset in kChromeAccentPresets)
                      _AccentSwatch(
                        preset: preset,
                        selected: colorsEqual(preset.color, controller.accent),
                        onTap: () => controller.setAccent(preset.color),
                      ),
                  ],
                ),
              ],
            );
          },
        ),
      ),
      actions: [
        TextButton(
          key: const Key('app_settings_close'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Close'),
        ),
      ],
    );
  }
}

class _AccentSwatch extends StatelessWidget {
  const _AccentSwatch({
    required this.preset,
    required this.selected,
    required this.onTap,
  });

  final ChromeAccentPreset preset;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      key: Key('chrome_accent_${preset.id}'),
      onTap: onTap,
      borderRadius: BorderRadius.circular(8),
      child: SizedBox(
        width: 72,
        child: Column(
          children: [
            Container(
              width: 36,
              height: 36,
              decoration: BoxDecoration(
                color: preset.color,
                shape: BoxShape.circle,
                border: Border.all(
                  color: selected ? Colors.black87 : Colors.black12,
                  width: selected ? 2.5 : 1,
                ),
                boxShadow: [
                  BoxShadow(
                    color: preset.color.withValues(alpha: 0.35),
                    blurRadius: 6,
                    offset: const Offset(0, 2),
                  ),
                ],
              ),
              child: selected
                  ? const Icon(Icons.check, size: 18, color: Colors.white)
                  : null,
            ),
            const SizedBox(height: 6),
            Text(
              preset.label,
              textAlign: TextAlign.center,
              style: WordTheme.ribbonLabel.copyWith(
                fontSize: 10,
                fontWeight: selected ? FontWeight.w600 : FontWeight.w400,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
