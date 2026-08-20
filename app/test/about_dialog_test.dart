import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/ui/about_dialog.dart';
import 'package:tutuaword/ui/privacy_policy_dialog.dart';

Future<void> _openAbout(WidgetTester tester) async {
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: Builder(
          builder: (context) => TextButton(
            onPressed: () => TutuawordAboutDialog.show(context),
            child: const Text('open'),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('open'));
  await tester.pumpAndSettle();
}

void main() {
  group('About and privacy dialogs', () {
    testWidgets('About dialog shows support email and links', (tester) async {
      await _openAbout(tester);

      expect(find.byKey(const Key('about_dialog')), findsOneWidget);
      expect(find.byKey(const Key('about_version')), findsOneWidget);
      expect(find.text('Version 26.08.19 (260819)'), findsOneWidget);
      expect(find.text('amit.blr76@gmail.com'), findsWidgets);
      expect(find.byKey(const Key('about_contact_support')), findsOneWidget);
      expect(find.byKey(const Key('about_keyboard_help')), findsOneWidget);
    });

    testWidgets('macOS About shows App Store rate, not Google Play', (tester) async {
      debugDefaultTargetPlatformOverride = TargetPlatform.macOS;
      try {
        await _openAbout(tester);

        expect(find.byKey(const Key('about_app_store_feedback')), findsOneWidget);
        expect(find.byKey(const Key('about_play_store_feedback')), findsNothing);
      } finally {
        debugDefaultTargetPlatformOverride = null;
      }
    });

    testWidgets('Android About shows Google Play rate, not App Store', (tester) async {
      debugDefaultTargetPlatformOverride = TargetPlatform.android;
      try {
        await _openAbout(tester);

        expect(find.byKey(const Key('about_play_store_feedback')), findsOneWidget);
        expect(find.byKey(const Key('about_app_store_feedback')), findsNothing);
      } finally {
        debugDefaultTargetPlatformOverride = null;
      }
    });

    testWidgets('Privacy policy dialog loads bundled policy', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) => TextButton(
                onPressed: () => PrivacyPolicyDialog.show(context),
                child: const Text('policy'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('policy'));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('privacy_policy_dialog')), findsOneWidget);
      expect(find.textContaining('Privacy Policy'), findsWidgets);
      expect(find.textContaining('amit.blr76@gmail.com'), findsOneWidget);
      expect(find.byKey(const Key('privacy_policy_open_online')), findsOneWidget);
      expect(
        find.textContaining('amit-tutu22.github.io/privacy-policy/tutuaword'),
        findsWidgets,
      );
    });
  });
}
