import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/main.dart';
import 'package:tutuaword/ui/info_bar.dart';
import 'package:tutuaword/ui/ribbon.dart';
import 'package:tutuaword/ui/ribbon_tabs/design_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/view_tab.dart';
import 'package:tutuaword/ui/status_bar.dart';
import 'package:tutuaword/ui/title_bar.dart';
import 'package:tutuaword/ui/word_theme.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  Future<void> pumpWide(WidgetTester tester, Widget child) async {
    await tester.binding.setSurfaceSize(const Size(1400, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(MaterialApp(home: Scaffold(body: child)));
  }

  group('Word shell integration', () {
    late EditorController controller;

    setUp(() {
      controller = EditorController();
    });

    tearDown(() {
      controller.dispose();
    });

    testWidgets('EditorScreen renders title bar, ribbon, canvas, and status bar', (tester) async {
      await pumpWide(tester, const EditorScreen());

      expect(find.byType(WordTitleBar), findsOneWidget);
      expect(find.byType(WordRibbon), findsOneWidget);
      expect(find.byType(WordStatusBar), findsOneWidget);
      expect(find.text('Document1'), findsOneWidget);
      expect(find.text('Home'), findsOneWidget);
      expect(find.text('Insert'), findsOneWidget);
      expect(find.text('View'), findsOneWidget);
    });

    testWidgets('InfoBar shows mock mode notice when engine is disconnected', (tester) async {
      if (controller.isEngineConnected) return;

      await pumpWide(tester, InfoBar(controller: controller));

      expect(
        find.textContaining('mock mode'),
        findsOneWidget,
      );
    });

    testWidgets('Title bar displays default document title', (tester) async {
      await pumpWide(tester, WordTitleBar(controller: controller));

      expect(find.text('Document1'), findsOneWidget);
    });
  });

  group('Ribbon tab integration', () {
    late EditorController controller;

    setUp(() {
      controller = EditorController();
    });

    tearDown(() {
      controller.dispose();
    });

    testWidgets('all eight ribbon tabs are present in tab strip', (tester) async {
      await pumpWide(
        tester,
        WordRibbon(controller: controller),
      );

      for (final label in [
        'Home',
        'Insert',
        'Design',
        'Layout',
        'References',
        'Mailings',
        'Review',
        'View',
      ]) {
        expect(find.text(label), findsOneWidget);
      }
    });

    testWidgets('Review tab toggles track changes', (tester) async {
      expect(controller.trackChanges, isFalse);

      await pumpWide(tester, ReviewTab(controller: controller));

      await tester.tap(find.text('Track\nChanges'));
      await tester.pump();

      expect(controller.trackChanges, isTrue);
    });

    testWidgets('View tab navigation pane checkbox toggles showNavigationPane', (tester) async {
      expect(controller.showNavigationPane, isFalse);

      await pumpWide(tester, ViewTab(controller: controller));

      await tester.tap(find.text('Navigation\nPane'));
      await tester.pump();

      expect(controller.showNavigationPane, isTrue);
    });

    testWidgets('Design tab renders theme gallery cards', (tester) async {
      await pumpWide(tester, const DesignTab());

      expect(find.text('Office'), findsOneWidget);
      expect(find.text('Document Formatting'), findsOneWidget);
    });

    testWidgets('Home tab italic and underline toggles update controller', (tester) async {
      await pumpWide(tester, HomeTab(controller: controller));

      await tester.tap(find.byIcon(Icons.format_italic));
      await tester.pump();
      expect(controller.italic, isTrue);

      await tester.tap(find.byIcon(Icons.format_underline));
      await tester.pump();
      expect(controller.underline, isTrue);
    });

    testWidgets('Home tab alignment buttons update controller', (tester) async {
      await pumpWide(tester, HomeTab(controller: controller));

      await tester.tap(find.byIcon(Icons.format_align_center));
      await tester.pump();
      expect(controller.alignment, TextAlign.center);

      await tester.tap(find.byIcon(Icons.format_align_right));
      await tester.pump();
      expect(controller.alignment, TextAlign.right);
    });

    testWidgets('Status bar zoom slider updates controller zoom', (tester) async {
      await pumpWide(tester, WordStatusBar(controller: controller));

      final slider = find.byType(Slider);
      expect(slider, findsOneWidget);

      await tester.drag(slider, const Offset(50, 0));
      await tester.pump();

      expect(controller.zoom, greaterThan(1.0));
    });

    testWidgets('Switching tabs via ribbon strip updates visible content', (tester) async {
      await pumpWide(tester, WordRibbon(controller: controller));

      expect(find.text('Paste'), findsOneWidget);

      await tester.tap(find.text('Review'));
      await tester.pump();

      expect(find.text('Spelling &\nGrammar'), findsOneWidget);
      expect(find.text('Paste'), findsNothing);
    });
  });

  group('WordTheme tokens', () {
    test('palette values match Word for Mac defaults', () {
      expect(WordTheme.titleBarBlue, const Color(0xFF2B579A));
      expect(WordTheme.canvasGray, const Color(0xFFE6E6E6));
      expect(WordTheme.ribbonHeight, 92);
      expect(WordTheme.titleBarHeight, 38);
      expect(WordTheme.statusBarHeight, 26);
    });
  });
}
