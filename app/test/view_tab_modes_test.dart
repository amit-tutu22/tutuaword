import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/document_view_layout.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_tabs/view_tab.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

import 'editor_test_helpers.dart';

Future<void> pumpViewTab(WidgetTester tester, EditorController controller) async {
  await tester.binding.setSurfaceSize(const Size(1600, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await pumpRibbonTab(
    tester,
    ViewTab(controller: controller),
    size: const Size(1600, 140),
  );
}

void main() {
  group('View tab modes and window controls', () {
    testWidgets('Read Mode / Web Layout / Print Layout switch', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpViewTab(tester, controller);

      await tester.tap(find.byKey(const Key('view_read_mode')));
      await tester.pump();
      expect(controller.viewLayout, DocumentViewLayout.readMode);
      expect(controller.isPageEditable(0), isFalse);
      expect(controller.statusText, contains('Read mode'));

      await tester.tap(find.byKey(const Key('view_web_layout')));
      await tester.pump();
      expect(controller.viewLayout, DocumentViewLayout.webLayout);
      expect(controller.isPageEditable(0), isTrue);

      await tester.tap(find.byKey(const Key('view_print_layout')));
      await tester.pump();
      expect(controller.viewLayout, DocumentViewLayout.printLayout);
      expect(controller.printPreview, isFalse);
    });

    testWidgets('Zoom dialog and page fit controls', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.reportViewportSize(const Size(800, 600));

      await pumpViewTab(tester, controller);

      await tester.tap(find.byKey(const Key('view_zoom')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('zoom_dialog')), findsOneWidget);
      await tester.tap(find.byKey(const Key('zoom_preset_150')));
      await tester.tap(find.byKey(const Key('zoom_ok')));
      await tester.pumpAndSettle();
      expect(controller.zoom, closeTo(1.5, 0.001));

      await tester.tap(find.byKey(const Key('view_one_page')));
      await tester.pump();
      expect(controller.pageColumns, 1);

      await tester.tap(find.byKey(const Key('view_multiple_pages')));
      await tester.pump();
      expect(controller.pageColumns, 2);
    });

    testWidgets('Split and Arrange All enable split view', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpViewTab(tester, controller);

      expect(controller.splitView, isFalse);
      await tester.tap(find.byKey(const Key('view_split')));
      await tester.pump();
      expect(controller.splitView, isTrue);

      await tester.tap(find.byKey(const Key('view_split')));
      await tester.pump();
      expect(controller.splitView, isFalse);

      await tester.tap(find.byKey(const Key('view_arrange_all')));
      await tester.pump();
      expect(controller.splitView, isTrue);
      expect(controller.statusText, contains('Arranged'));
    });

    testWidgets('New Window opens secondary view', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.binding.setSurfaceSize(const Size(1600, 900));
      addTearDown(() => tester.binding.setSurfaceSize(null));
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: ViewTab(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byKey(const Key('view_new_window')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('secondary_document_window')), findsOneWidget);

      await tester.tap(find.byKey(const Key('secondary_window_close')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('secondary_document_window')), findsNothing);
    });

    testWidgets('no Coming soon tooltips on wired View controls', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpViewTab(tester, controller);
      expect(find.byTooltip(kComingSoonTooltip), findsNothing);
    });
  });
}
