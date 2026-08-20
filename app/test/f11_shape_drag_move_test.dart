import 'dart:ui';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';
import 'package:tutuaword/editor/shape_hit_test.dart';

import 'editor_test_helpers.dart';

void main() {
  group('Inserted object drag move', () {
    testWidgets('locks document scroll while shape move is active', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.selectDiagram(
        0,
        const ShapeBounds(
          shapeId: '00000000-0000-0000-0000-00000000d001',
          index: 0,
          rect: Rect.fromLTWH(72, 72, 200, 100),
        ),
      );
      expect(controller.locksDocumentScroll, isFalse);
      controller.beginShapeMove(const Offset(80, 80));
      expect(controller.locksDocumentScroll, isTrue);
      controller.cancelShapeMove();
      expect(controller.locksDocumentScroll, isFalse);
    });

    testWidgets('iOS vertical drag moves shape without scrolling pages', (tester) async {
      debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
      addTearDown(() {
        debugDefaultTargetPlatformOverride = null;
      });
      tester.view.physicalSize = const Size(1024, 1366);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.reset);

      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 4);
      controller.selectDiagram(
        0,
        const ShapeBounds(
          shapeId: '00000000-0000-0000-0000-00000000d001',
          index: 0,
          rect: Rect.fromLTWH(72, 72, 200, 100),
        ),
      );

      await pumpTestDocumentView(tester, controller);

      final scrollable = tester.state<ScrollableState>(find.byType(Scrollable).first);
      final startOffset = scrollable.position.pixels;
      final surface = find.byType(GlyphEditorSurface).first;
      final box = tester.renderObject(surface) as RenderBox;
      final gesture = await tester.startGesture(
        box.localToGlobal(const Offset(100, 100)),
      );
      await tester.pump();
      expect(controller.locksDocumentScroll, isTrue);

      // moveBy is in global pixels; phone canvas is FittedBox-scaled, so aim
      // with localToGlobal so the page-space delta is exactly 90.
      await gesture.moveTo(box.localToGlobal(const Offset(100, 190)));
      await tester.pump();

      expect(controller.contentDragOffset.dy, closeTo(90, 1));
      expect(scrollable.position.pixels, startOffset);

      await gesture.up();
      await settleEngineStyle(tester);
      expect(engine.lastShapeAnchorId, '00000000-0000-0000-0000-00000000d001');
      expect(engine.lastShapeAnchorY, closeTo(90, 1));
      debugDefaultTargetPlatformOverride = null;
    });

    testWidgets('commits shape anchor via engine', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.insertSmartArt();
      await settleEngineStyle(tester);

      controller.selectDiagram(
        0,
        const ShapeBounds(
          shapeId: '00000000-0000-0000-0000-00000000d001',
          index: 0,
          rect: Rect.fromLTWH(72, 72, 200, 100),
        ),
      );
      controller.beginShapeMove(const Offset(80, 80));
      controller.updateShapeMove(const Offset(120, 100));
      await controller.commitShapeMove();
      await settleEngineStyle(tester);

      expect(engine.lastShapeAnchorId, '00000000-0000-0000-0000-00000000d001');
      expect(engine.lastShapeAnchorX, 40);
      expect(engine.lastShapeAnchorY, 20);
      expect(controller.sessionController.statusText, contains('moved'));
      expect(controller.contentDragOffset, Offset.zero);
    });

    testWidgets('commits table anchor via engine', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      const tableId = '00000000-0000-0000-0000-00000000t001';
      controller.selectDiagram(
        0,
        const ShapeBounds(
          shapeId: tableId,
          index: 0,
          rect: Rect.fromLTWH(72, 72, 300, 120),
        ),
      );
      controller.beginShapeMove(const Offset(80, 80));
      controller.updateShapeMove(const Offset(110, 110));
      await controller.commitShapeMove();
      await settleEngineStyle(tester);

      expect(engine.lastShapeAnchorId, tableId);
      expect(engine.lastShapeAnchorX, 30);
      expect(engine.lastShapeAnchorY, 30);
      expect(controller.sessionController.statusText, contains('moved'));
    });

    testWidgets('live drag offset follows preview before commit', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.selectDiagram(
        0,
        const ShapeBounds(
          shapeId: '00000000-0000-0000-0000-00000000d001',
          index: 0,
          rect: Rect.fromLTWH(72, 72, 200, 100),
        ),
      );
      controller.beginShapeMove(const Offset(80, 80));
      controller.updateShapeMove(const Offset(120, 100));

      expect(controller.contentDragSourceRect, const Rect.fromLTWH(72, 72, 200, 100));
      expect(controller.contentDragOffset, const Offset(40, 20));
      expect(controller.selectedDiagramRect, const Rect.fromLTWH(112, 92, 200, 100));
    });
  });
}
