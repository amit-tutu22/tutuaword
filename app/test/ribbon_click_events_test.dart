import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/layout_tab.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

import 'editor_test_helpers.dart';

/// UI pointer-click coverage for ribbon controls (F21 regression).
///
/// These tests use [tester.tap] (mouse path), not keyboard ActivateIntent.
/// They fail if [RibbonFocusable] drops GestureDetector / onTap wiring.
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  Map<String, dynamic> engineCharFormat(MockDocumentEngine engine) {
    final json = jsonDecode(engine.fetchCaretFormat(engine.defaultRunId)!)
        as Map<String, dynamic>;
    return json['char_format'] as Map<String, dynamic>;
  }

  group('Ribbon pointer click events', () {
    testWidgets('U-ribbon-click-bold-italic-underline', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      await typeTextDirect(controller, 'click');

      await pumpWideRibbon(
        tester,
        SizedBox(height: 140, child: HomeTab(controller: controller)),
      );

      await tester.tap(find.byTooltip('Bold'));
      await controller.ensureLayoutReady();
      await tester.pump();
      expect(controller.bold, isTrue);
      expect(engineCharFormat(engine)['bold'], isTrue);

      await tester.tap(find.byTooltip('Italic'));
      await controller.ensureLayoutReady();
      await tester.pump();
      expect(controller.italic, isTrue);
      expect(engineCharFormat(engine)['italic'], isTrue);

      await tester.tap(find.byTooltip('Underline'));
      await controller.ensureLayoutReady();
      await tester.pump();
      expect(controller.underline, isTrue);
      expect(engineCharFormat(engine)['underline'], 'Single');
    });

    testWidgets('U-ribbon-click-alignment-center', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      await typeTextDirect(controller, 'align');

      await pumpWideRibbon(
        tester,
        SizedBox(height: 140, child: HomeTab(controller: controller)),
      );

      await tester.tap(find.byTooltip('Center'));
      await controller.ensureLayoutReady();
      await tester.pump();

      expect(controller.alignment, TextAlign.center);
      expect(mockEngineParaFormat(engine)['alignment'], 'Center');
    });

    testWidgets('U-ribbon-click-bullets', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      await typeTextDirect(controller, 'item');

      await pumpWideRibbon(
        tester,
        SizedBox(height: 140, child: HomeTab(controller: controller)),
      );

      await tester.tap(find.byTooltip('Bullets'));
      await controller.ensureLayoutReady();
      await tester.pump();

      expect(mockEngineNumbering(engine), isNotNull);
    });

    testWidgets('U-ribbon-click-heading-style-card', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      await typeTextDirect(controller, 'Title');

      await pumpWideRibbon(
        tester,
        SizedBox(height: 140, child: HomeTab(controller: controller)),
      );

      await tester.tap(find.text('Heading 1'));
      await settleEngineStyle(tester);

      expect(controller.activeParagraphStyle, 'Heading 1');
    });

    testWidgets('U-ribbon-click-tab-switch-insert-shows-table', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.binding.setSurfaceSize(const Size(1400, 900));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WordRibbon(controller: controller),
          ),
        ),
      );
      await tester.pump();

      expect(find.text('Table'), findsNothing);
      await tester.tap(find.text('Insert'));
      await tester.pump();
      expect(find.text('Table'), findsOneWidget);

      await tester.tap(find.text('Home'));
      await tester.pump();
      expect(find.text('Heading 1'), findsOneWidget);
    });

    testWidgets('U-ribbon-click-insert-table-large-button', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpWideRibbon(
        tester,
        SizedBox(height: 140, child: InsertTab(controller: controller)),
      );

      await tester.tap(find.byKey(const Key('insert_table_button')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('table_size_cell_2_3')));
      await settleEngineStyle(tester);

      expect(engine.hasTable, isTrue);
      expect(engine.tableRows, 2);
      expect(engine.tableCols, 3);
    });

    testWidgets('U-ribbon-click-layout-orientation', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await pumpWideRibbon(
        tester,
        SizedBox(height: 140, child: LayoutTab(controller: controller)),
      );

      expect(controller.isLandscape, isFalse);
      await tester.tap(find.byTooltip('Portrait'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Landscape').last);
      await settleEngineStyle(tester);
      expect(controller.isLandscape, isTrue);
    });

    testWidgets('U-ribbon-click-disabled-control-noop', (tester) async {
      var pressed = 0;
      await pumpWideRibbon(
        tester,
        RibbonToggleButton(
          icon: Icons.format_bold,
          tooltip: 'Disabled Bold',
          selected: false,
          onPressed: null,
        ),
      );

      // Disabled controls keep their tooltip; taps must not invoke a handler.
      await tester.tap(find.byTooltip('Disabled Bold'));
      await tester.pump();
      expect(pressed, 0);
    });
  });
}
