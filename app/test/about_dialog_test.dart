import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/ui/about_dialog.dart';
import 'package:tutuaword/ui/privacy_policy_dialog.dart';

void main() {
  group('About and privacy dialogs', () {
    testWidgets('About dialog shows support email and links', (tester) async {
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

      expect(find.byKey(const Key('about_dialog')), findsOneWidget);
      expect(find.byKey(const Key('about_version')), findsOneWidget);
      expect(find.text('Version 26.08.01 (260801)'), findsOneWidget);
      expect(find.text('amit.blr76@gmail.com'), findsWidgets);
      expect(find.byKey(const Key('about_contact_support')), findsOneWidget);
      expect(find.byKey(const Key('about_play_store_feedback')), findsOneWidget);
      expect(find.byKey(const Key('about_app_store_feedback')), findsOneWidget);
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
    });
  });
}
