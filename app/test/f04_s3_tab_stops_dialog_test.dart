import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_tabs/layout_tab.dart';
import 'package:tutuaword/ui/tab_stops_dialog.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F04.S3 Tab stops editor', () {
    testWidgets('I-F04-S3-tab-stops-dialog adds and clears stops', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'A');

      await pumpWideRibbon(
        tester,
        SizedBox(height: 120, child: LayoutTab(controller: controller)),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Tab Stops'));
      await tester.pumpAndSettle();
      expect(find.byType(TabStopsDialog), findsOneWidget);

      await tester.enterText(find.byKey(const Key('tab_stop_position')), '200');
      await tester.tap(find.byKey(const Key('tab_stop_alignment')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Right').last);
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('tab_stop_add')));
      await tester.pumpAndSettle();
      expect(find.textContaining('200 pt — Right'), findsOneWidget);

      await tester.tap(find.byKey(const Key('tab_stops_dialog_ok')));
      await tester.pumpAndSettle();
      await controller.ensureLayoutReady();

      expect(controller.tabStops, hasLength(1));
      expect((controller.tabStops.first['position'] as num).toDouble(), 200);
      expect(controller.tabStops.first['alignment'], 'Right');

      final stops = mockEngineParaFormat(engine)['tab_stops'] as List;
      expect(stops, hasLength(1));
      expect((stops.first as Map)['position'], 200);
      expect(stops.first['alignment'], 'Right');

      await tester.tap(find.byTooltip('Tab Stops'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('tab_stop_clear')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('tab_stops_dialog_ok')));
      await tester.pumpAndSettle();
      await controller.ensureLayoutReady();

      expect(controller.tabStops, isEmpty);
      expect(mockEngineParaFormat(engine)['tab_stops'], isEmpty);
    });
  });
}
