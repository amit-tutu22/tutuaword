import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/view_tab.dart';

import 'editor_test_helpers.dart';
import 'package:tutuaword/ui/status_bar.dart';
import 'package:tutuaword/ui/title_bar.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Word Ribbon UI', () {
    late EditorController controller;

    setUp(() {
      controller = EditorController.forTest();
    });

    tearDown(() {
      controller.dispose();
    });

    testWidgets('Home ribbon renders Clipboard, Font, Paragraph, and Styles groups', (tester) async {
      await pumpWideRibbon(tester, HomeTab(controller: controller));

      expect(find.text('Clipboard'), findsOneWidget);
      expect(find.text('Font'), findsOneWidget);
      expect(find.text('Paragraph'), findsOneWidget);
      expect(find.text('Styles'), findsOneWidget);
      expect(find.text('Paste'), findsOneWidget);
      expect(find.text('Heading 1'), findsOneWidget);
    });

    testWidgets('Tapping Bold button flips controller.bold', (tester) async {
      expect(controller.bold, isFalse);

      await pumpWideRibbon(tester, HomeTab(controller: controller));

      await tester.tap(find.byTooltip('Bold'));
      await tester.pump();
      await controller.ensureLayoutReady();

      expect(controller.bold, isTrue);
    });

    testWidgets('Switching to Insert tab shows Table button', (tester) async {
      await tester.binding.setSurfaceSize(const Size(1400, 900));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      final ribbonKey = GlobalKey<WordRibbonState>();

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Column(
              children: [
                WordRibbon(key: ribbonKey, controller: controller),
              ],
            ),
          ),
        ),
      );

      expect(find.text('Table'), findsNothing);

      await tester.tap(find.text('Insert'));
      await tester.pump();

      expect(find.text('Table'), findsOneWidget);
      expect(find.text('Heading 1'), findsNothing);
    });

    testWidgets('Status bar shows page indicator and word count', (tester) async {
      await tester.binding.setSurfaceSize(const Size(1400, 900));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      controller.replacePageText(0, 'Hello world test');

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WordStatusBar(controller: controller),
          ),
        ),
      );

      expect(find.text('Page 1 of 1'), findsOneWidget);
      expect(find.text('3 words'), findsOneWidget);
      expect(find.text('100%'), findsOneWidget);
    });

    testWidgets('View tab Ruler checkbox toggles controller.showRuler', (tester) async {
      expect(controller.showRuler, isFalse);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ViewTab(controller: controller),
          ),
        ),
      );

      await tester.tap(find.text('Ruler'));
      await tester.pump();

      expect(controller.showRuler, isTrue);
    });

    testWidgets('Insert tab renders Table button wired to controller', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: InsertTab(controller: controller),
          ),
        ),
      );

      expect(find.text('Table'), findsOneWidget);
      expect(find.text('Pictures'), findsOneWidget);
    });

    testWidgets('Title bar renders quick-access icons', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WordTitleBar(controller: controller),
          ),
        ),
      );

      expect(find.byIcon(Icons.save_outlined), findsOneWidget);
      expect(find.byIcon(Icons.undo), findsOneWidget);
      expect(find.byIcon(Icons.redo), findsOneWidget);
      expect(find.text('Document1'), findsOneWidget);
    });

    testWidgets('Review tab renders spelling and track changes controls', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ReviewTab(controller: controller),
          ),
        ),
      );

      expect(find.text('Spelling &\nGrammar'), findsOneWidget);
      expect(find.text('Track\nChanges'), findsOneWidget);
    });

    testWidgets('Italic toggle in Home tab updates controller', (tester) async {
      await pumpWideRibbon(tester, HomeTab(controller: controller));

      await tester.tap(find.byTooltip('Italic'));
      await tester.pump();
      await controller.ensureLayoutReady();
      expect(controller.italic, isTrue);
    });
  });
}
