import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/navigation_pane.dart';
import 'package:tutuaword/editor/page_navigator.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F19.S1 Thumbnails and page strip', () {
    testWidgets('I-F19-S1-page-nav-jump click page N scrolls canvas', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 3);
      controller.toggleNavigationPane();

      await tester.binding.setSurfaceSize(const Size(900, 700));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await pumpTestDocumentView(tester, controller);
      await tester.pumpAndSettle();

      expect(find.byType(NavigationPane), findsOneWidget);
      expect(find.byType(PageNavigator), findsOneWidget);
      expect(find.byKey(const ValueKey('page-thumbnail-0')), findsOneWidget);
      expect(find.byKey(const ValueKey('page-thumbnail-1')), findsOneWidget);
      expect(find.byKey(const ValueKey('page-thumbnail-2')), findsOneWidget);

      expect(controller.currentPage, 0);

      await tester.ensureVisible(find.byKey(const ValueKey('page-thumbnail-2')));
      await tester.tap(find.byKey(const ValueKey('page-thumbnail-2')));
      await tester.pump(); // process scroll request
      await tester.pumpAndSettle();

      expect(controller.currentPage, 2);

      final pageList = find.byKey(const ValueKey('document-page-list'));
      expect(pageList, findsOneWidget);
      final listView = tester.widget<ListView>(pageList);
      final offset = listView.controller?.offset ?? 0;
      expect(
        offset,
        greaterThan(0),
        reason: 'canvas should scroll to page 3 (index 2), got offset=$offset',
      );
    });

    test('jumpToPage clamps and requests scroll', () {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 4);

      controller.jumpToPage(99);
      expect(controller.currentPage, 3);
      expect(controller.view.scrollRequestPage, 3);

      controller.view.takeScrollRequest();
      controller.jumpToPage(1);
      expect(controller.currentPage, 1);
      expect(controller.view.takeScrollRequest(), 1);
    });
  });
}
