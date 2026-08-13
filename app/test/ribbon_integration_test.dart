import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/editor_screen.dart';
import 'package:tutuaword/ui/info_bar.dart';
import 'package:tutuaword/ui/ribbon.dart';
import 'package:tutuaword/ui/ribbon_tabs/design_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/mailings_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/references_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/view_tab.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';
import 'package:tutuaword/ui/status_bar.dart';
import 'package:tutuaword/ui/title_bar.dart';
import 'package:tutuaword/ui/word_theme.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  Future<void> pumpWide(WidgetTester tester, Widget child) async {
    await tester.binding.setSurfaceSize(const Size(1400, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      MaterialApp(
        home: MediaQuery(
          data: const MediaQueryData(size: Size(1400, 900)),
          child: Scaffold(body: child),
        ),
      ),
    );
  }

  Future<void> runOnPhone(WidgetTester tester, Future<void> Function() body) async {
    debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
    await tester.binding.setSurfaceSize(const Size(390, 844));
    try {
      await body();
    } finally {
      debugDefaultTargetPlatformOverride = null;
      await tester.binding.setSurfaceSize(null);
    }
  }

  Future<void> runOnTablet(WidgetTester tester, Future<void> Function() body) async {
    debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
    await tester.binding.setSurfaceSize(const Size(820, 1180));
    try {
      await body();
    } finally {
      debugDefaultTargetPlatformOverride = null;
      await tester.binding.setSurfaceSize(null);
    }
  }

  Future<void> pumpPhone(WidgetTester tester, Widget child) async {
    await tester.pumpWidget(
      MaterialApp(
        home: MediaQuery(
          data: const MediaQueryData(size: Size(390, 844)),
          child: Scaffold(body: child),
        ),
      ),
    );
  }

  Future<void> pumpTablet(WidgetTester tester, Widget child) async {
    await tester.pumpWidget(
      MaterialApp(
        home: MediaQuery(
          data: const MediaQueryData(size: Size(820, 1180)),
          child: Scaffold(body: child),
        ),
      ),
    );
  }

  group('Word shell integration', () {
    late EditorController controller;

    setUp(() {
      controller = EditorController.forTest();
      controller.ensureGlyphCaret();
    });

    tearDown(() {
      controller.dispose();
    });

    testWidgets('EditorScreen renders title bar, ribbon, canvas, and status bar', (tester) async {
      await pumpWide(tester, EditorScreen(controller: controller));

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

    testWidgets('Title bar Home icon switches to Home ribbon tab', (tester) async {
      final ribbonKey = GlobalKey<WordRibbonState>();
      await pumpWide(
        tester,
        Column(
          children: [
            WordTitleBar(
              controller: controller,
              onHomePressed: () =>
                  ribbonKey.currentState?.selectTab(RibbonTab.home),
            ),
            WordRibbon(key: ribbonKey, controller: controller),
          ],
        ),
      );

      await tester.tap(find.text('Insert'));
      await tester.pump();
      expect(find.byType(HomeTab), findsNothing);

      await tester.tap(find.byIcon(Icons.home_outlined));
      await tester.pump();
      expect(find.byType(HomeTab), findsOneWidget);
    });

    testWidgets('Title bar displays default document title', (tester) async {
      await pumpWide(tester, WordTitleBar(controller: controller));

      expect(find.text('Document1'), findsOneWidget);
    });

    testWidgets('Title bar Open control is enabled for mobile import', (tester) async {
      await pumpWide(tester, WordTitleBar(controller: controller));

      final openIcon = find.byIcon(Icons.folder_open_outlined);
      expect(openIcon, findsOneWidget);
      // Tooltip wraps the gesture target; long-press surfaces the label.
      await tester.longPress(openIcon);
      await tester.pumpAndSettle();
      expect(find.text('Open'), findsOneWidget);
    });

    testWidgets('Title bar centers document name on the full bar width', (tester) async {
      debugDefaultTargetPlatformOverride = TargetPlatform.macOS;
      try {
        await pumpWide(tester, WordTitleBar(controller: controller));

        final title = tester.getCenter(find.text('Document1'));
        final bar = tester.getRect(find.byType(WordTitleBar));
        expect(title.dx, closeTo(bar.center.dx, 1.0));
      } finally {
        debugDefaultTargetPlatformOverride = null;
        await tester.binding.setSurfaceSize(null);
      }
    });

    testWidgets('Compact title bar keeps document name between icon groups', (tester) async {
      await runOnPhone(tester, () async {
        await pumpPhone(tester, WordTitleBar(controller: controller));

        expect(find.text('Document1'), findsOneWidget);
        expect(find.byIcon(Icons.undo), findsNothing);
        expect(find.byIcon(Icons.print_outlined), findsNothing);
        expect(find.byIcon(Icons.search), findsOneWidget);
        expect(find.byKey(const Key('title_bar_overflow')), findsOneWidget);
      });
    });

    testWidgets('Tablet title bar shows full quick-access toolbar', (tester) async {
      await runOnTablet(tester, () async {
        await pumpTablet(tester, WordTitleBar(controller: controller));

        expect(find.text('Document1'), findsOneWidget);
        expect(find.byIcon(Icons.undo), findsOneWidget);
        expect(find.byIcon(Icons.print_outlined), findsOneWidget);
        expect(find.byKey(const Key('title_bar_overflow')), findsNothing);
      });
    });
  });

  group('Ribbon tab integration', () {
    late EditorController controller;

    setUp(() {
      controller = EditorController.forTest();
      controller.ensureGlyphCaret();
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

      await tester.tap(find.byKey(const Key('view_show_navigation_pane')));
      await tester.pump();

      expect(controller.showNavigationPane, isTrue);
    });

    testWidgets('Design tab renders theme gallery cards', (tester) async {
      await pumpWide(tester, DesignTab(controller: controller));

      expect(find.text('Office'), findsOneWidget);
      expect(find.text('Document Formatting'), findsOneWidget);
    });

    testWidgets('Home tab font dropdown shows choices and updates controller', (tester) async {
      await pumpWide(tester, HomeTab(controller: controller));

      expect(find.text('Calibri'), findsOneWidget);
      await tester.tap(find.text('Calibri'));
      await tester.pumpAndSettle();

      expect(find.text('Arial'), findsOneWidget);
      expect(find.text('Times New Roman'), findsOneWidget);

      await tester.tap(find.text('Arial').last);
      await tester.pumpAndSettle();
      await controller.ensureLayoutReady();

      expect(controller.fontFamily, 'Arial');
    });

    testWidgets('Home tab font size dropdown shows choices and updates controller', (tester) async {
      await pumpWide(tester, HomeTab(controller: controller));

      await tester.tap(find.text('11'));
      await tester.pumpAndSettle();

      expect(find.text('14'), findsOneWidget);
      expect(find.text('12'), findsOneWidget);

      await tester.tap(find.text('14').last);
      await tester.pumpAndSettle();
      await controller.ensureLayoutReady();

      expect(controller.fontSize, 14);
    });

    testWidgets('Home tab font size dropdown displays picked size in ribbon', (tester) async {
      await pumpWide(tester, HomeTab(controller: controller));

      await tester.tap(find.text('11'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('14').last);
      await tester.pumpAndSettle();
      await controller.ensureLayoutReady();

      expect(controller.fontSize, 14);
      expect(find.text('14'), findsOneWidget);
      expect(find.text('11'), findsNothing);
    });

    testWidgets('Home tab exposes F03.S4 font effect toggles', (tester) async {
      await pumpWide(tester, HomeTab(controller: controller));

      expect(find.byTooltip('All Caps'), findsOneWidget);
      expect(find.byTooltip('Small Caps'), findsOneWidget);
      expect(find.byTooltip('Hidden'), findsOneWidget);
      expect(find.byTooltip('Ligatures'), findsOneWidget);

      await tester.tap(find.byTooltip('All Caps'));
      await tester.pump();
      await controller.ensureLayoutReady();
      expect(controller.allCaps, isTrue);

      await tester.tap(find.byTooltip('Hidden'));
      await tester.pump();
      await controller.ensureLayoutReady();
      expect(controller.hidden, isTrue);
    });

    testWidgets('Home tab italic and underline toggles update controller', (tester) async {
      await pumpWide(tester, HomeTab(controller: controller));

      await tester.tap(find.byIcon(Icons.format_italic));
      await tester.pump();
      await controller.ensureLayoutReady();
      expect(controller.italic, isTrue);

      await tester.tap(find.byIcon(Icons.format_underline));
      await tester.pump();
      await controller.ensureLayoutReady();
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

    testWidgets('Phone status bar hides wide slider and view mode icons', (tester) async {
      await runOnPhone(tester, () async {
        await pumpPhone(tester, WordStatusBar(controller: controller));

        expect(find.byType(Slider), findsNothing);
        expect(find.byIcon(Icons.article_outlined), findsNothing);
        expect(find.byKey(const Key('status_view_menu')), findsOneWidget);
        expect(find.textContaining('Page 1 of'), findsOneWidget);
      });
    });

    testWidgets('Phone ribbon shows icon-only Share button', (tester) async {
      await runOnPhone(tester, () async {
        await pumpPhone(tester, WordRibbon(controller: controller));

        expect(find.byKey(const Key('ribbon_share')), findsOneWidget);
        expect(find.text('Share'), findsNothing);
      });
    });

    testWidgets('Tablet ribbon keeps Share label', (tester) async {
      await runOnTablet(tester, () async {
        await pumpTablet(tester, WordRibbon(controller: controller));

        expect(find.text('Share'), findsOneWidget);
      });
    });

    testWidgets('Design tab has no Coming soon placeholders', (tester) async {
      await pumpWide(tester, DesignTab(controller: controller));

      expect(find.byTooltip(kComingSoonTooltip), findsNothing);
    });

    testWidgets('Review tab shows Export PDF and accept/reject actions', (tester) async {
      await pumpWide(tester, ReviewTab(controller: controller));

      expect(find.text('Export\nPDF'), findsOneWidget);
      expect(find.byTooltip(kTrackChangeAcceptTooltip), findsOneWidget);
      expect(find.byTooltip(kTrackChangeRejectTooltip), findsOneWidget);
    });

    testWidgets('Review tab Accept All and Reject All update mock document', (tester) async {
      final engine = MockDocumentEngine(initialText: '');
      final tc = EditorController.forTest(engine: engine);
      addTearDown(tc.dispose);
      tc.toggleTrackChanges();
      await engine.tryInsertTextAsync(engine.defaultRunId, 0, 'Tracked');

      await pumpWide(tester, ReviewTab(controller: tc));
      expect(find.byKey(const Key('accept_all_revisions')), findsOneWidget);
      expect(find.byKey(const Key('reject_all_revisions')), findsOneWidget);

      tc.acceptAllRevisions();
      expect(tc.statusText, contains('Accepted all revisions'));

      await engine.tryInsertTextAsync(engine.defaultRunId, 0, 'Again');
      tc.rejectAllRevisions();
      expect(tc.statusText, contains('Rejected all revisions'));
    });

    testWidgets('Mailings tab wires Next Record field insert', (tester) async {
      await pumpWide(tester, MailingsTab(controller: controller));
      expect(find.byKey(const Key('insert_next_record')), findsOneWidget);
    });

    testWidgets('References tab shows endnote and table of figures', (tester) async {
      await pumpWide(tester, ReferencesTab(controller: controller));

      expect(find.byKey(const Key('insert_endnote')), findsOneWidget);
      expect(find.byKey(const Key('insert_table_of_figures')), findsOneWidget);
      expect(find.byKey(const Key('insert_table_of_contents')), findsOneWidget);
    });

    testWidgets('Review tab shows smart assist controls', (tester) async {
      await pumpWide(tester, ReviewTab(controller: controller));

      expect(find.byKey(const Key('review_consistency')), findsOneWidget);
      expect(find.byKey(const Key('review_audience_rewrite')), findsOneWidget);
      expect(find.byKey(const Key('review_auto_alt_text')), findsOneWidget);
    });

    testWidgets('I-F01-S3-print-preview-toggle from View tab', (tester) async {
      await pumpWide(tester, ViewTab(controller: controller));

      expect(controller.printPreview, isFalse);
      await tester.tap(find.text('Print\nPreview'));
      await tester.pump();
      expect(controller.printPreview, isTrue);
      expect(controller.statusText, contains('Print preview'));

      await tester.tap(find.text('Print\nLayout'));
      await tester.pump();
      expect(controller.printPreview, isFalse);
      expect(controller.statusText, contains('Print layout'));
    });

    testWidgets('View tab Read Mode and Split are wired', (tester) async {
      await pumpWide(tester, ViewTab(controller: controller));

      await tester.tap(find.text('Read\nMode'));
      await tester.pump();
      expect(controller.isReadMode, isTrue);

      await tester.tap(find.text('Print\nLayout'));
      await tester.pump();
      expect(controller.isReadMode, isFalse);

      await tester.tap(find.text('Split'));
      await tester.pump();
      expect(controller.splitView, isTrue);
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
      expect(WordTheme.ribbonHeightPhone, 64);
      expect(WordTheme.titleBarHeight, 38);
      expect(WordTheme.statusBarHeight, 26);
    });

    testWidgets('Layout tab fits phone ribbon height', (tester) async {
      await runOnPhone(tester, () async {
        final controller = createTestEditorController(
          engine: MockDocumentEngine(),
        );
        addTearDown(controller.dispose);
        await pumpPhone(tester, WordRibbon(controller: controller));
        await tester.pumpAndSettle();

        await tester.tap(find.text('Layout'));
        await tester.pumpAndSettle();

        expect(tester.takeException(), isNull);
        expect(find.textContaining('OVERFLOWED'), findsNothing);
        // Phone chrome hides icon-button labels inside the 64px ribbon.
        expect(find.text('Before'), findsNothing);
        expect(find.text('After'), findsNothing);
      });
    });

    testWidgets('phoneChrome and tabletChrome breakpoints', (tester) async {
      debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
      try {
        await tester.pumpWidget(
          MaterialApp(
            home: MediaQuery(
              data: const MediaQueryData(size: Size(390, 844)),
              child: Builder(
                builder: (context) {
                  expect(WordTheme.phoneChrome(context), isTrue);
                  expect(WordTheme.tabletChrome(context), isFalse);
                  expect(WordTheme.leadingChromeInset(context), 8.0);
                  expect(WordTheme.ribbonHeightFor(context), WordTheme.ribbonHeightPhone);
                  return const SizedBox();
                },
              ),
            ),
          ),
        );

        await tester.binding.setSurfaceSize(const Size(820, 1180));
        await tester.pumpWidget(
          MaterialApp(
            home: MediaQuery(
              data: const MediaQueryData(size: Size(820, 1180)),
              child: Builder(
                builder: (context) {
                  expect(WordTheme.phoneChrome(context), isFalse);
                  expect(WordTheme.tabletChrome(context), isTrue);
                  expect(WordTheme.ribbonHeightFor(context), WordTheme.ribbonHeight);
                  return const SizedBox();
                },
              ),
            ),
          ),
        );
      } finally {
        debugDefaultTargetPlatformOverride = null;
        await tester.binding.setSurfaceSize(null);
      }
    });
  });
}
