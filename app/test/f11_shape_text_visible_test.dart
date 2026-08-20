import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';
import 'package:tutuaword/editor/shape_hit_test.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';

import 'editor_test_helpers.dart';

const _caret = Key('glyph_caret');
const _handles = Key('shape_selection_handles');

const _wordArtBounds = ShapeBounds(
  shapeId: 'wordart-visible-1',
  index: 0,
  rect: Rect.fromLTWH(72, 72, 240, 72),
);

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('I-F11-visible-caret-hides-when-wordart-is-selected',
      (tester) async {
    final env = await _pumpCanvas(tester);

    await typeText(tester, env.controller, 'ab');
    expect(find.byKey(_caret), findsOneWidget);
    expect(find.byKey(_handles), findsNothing);

    env.controller.selectDiagram(0, _wordArtBounds);
    await tester.pump();

    expect(env.controller.hasSelectedDiagram, isTrue);
    expect(env.controller.caretGeometry, isNull);
    expect(find.byKey(_caret), findsNothing);
    expect(find.byKey(_handles), findsOneWidget);
  });

  testWidgets('I-F11-visible-typing-into-selected-wordart-shows-caret',
      (tester) async {
    final env = await _pumpCanvas(tester);
    env.engine.lastSplitCaret = HitTestResult(
      runId: env.engine.defaultRunId,
      charOffset: 0,
    );

    env.controller.selectDiagram(0, _wordArtBounds);
    await tester.pump();
    expect(find.byKey(_caret), findsNothing);
    expect(find.byKey(_handles), findsOneWidget);

    await typeText(tester, env.controller, 'hi');

    expect(env.controller.hasSelectedDiagram, isFalse);
    expect(env.engine.text, contains('hi'));
    expect(env.controller.caretGeometry, isNotNull);
    expect(find.byKey(_handles), findsNothing);
    expect(find.byKey(_caret), findsOneWidget);
  });

  testWidgets('I-F11-visible-second-click-on-shape-shows-caret', (tester) async {
    final env = await _pumpCanvas(
      tester,
      displayList: _shapeDisplayList(
        shapeId: _wordArtBounds.shapeId,
        rect: _wordArtBounds.rect,
      ),
    );

    await _tapShapeInterior(tester);
    await tester.pump();
    expect(env.controller.hasSelectedDiagram, isTrue);
    expect(find.byKey(_handles), findsOneWidget);
    expect(find.byKey(_caret), findsNothing);

    await _tapShapeInterior(tester);
    await settleEngineStyle(tester);

    expect(env.controller.hasSelectedDiagram, isFalse);
    expect(find.byKey(_handles), findsNothing);
    expect(find.byKey(_caret), findsOneWidget);
  });

  testWidgets('I-F11-visible-insert-text-box-then-type-without-clicking-canvas',
      (tester) async {
    final env = await _pumpRibbonAndCanvas(tester);

    await tester.tap(find.byKey(const Key('insert_text_box')));
    await settleEngineStyle(tester);

    // Ribbon must not keep the keys — type without tapping the page.
    await typeText(tester, env.controller, 'ab');

    expect(env.engine.text, contains('ab'));
    expect(find.byKey(_caret), findsOneWidget);
  });

  testWidgets('I-F11-visible-insert-word-art-dialog-then-type', (tester) async {
    final env = await _pumpRibbonAndCanvas(tester);

    await tester.tap(find.byKey(const Key('insert_word_art')));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(TextButton, 'Insert'));
    await settleEngineStyle(tester);

    await typeText(tester, env.controller, 'xy');

    expect(env.controller.sessionController.statusText, contains('WordArt'));
    expect(env.engine.text, contains('xy'));
    expect(find.byKey(_caret), findsOneWidget);
  });

  testWidgets('I-F11-visible-insert-rectangle-then-type', (tester) async {
    final env = await _pumpRibbonAndCanvas(tester);

    await tester.tap(find.byKey(const Key('insert_shapes')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('shape_picker_rectangle')));
    await settleEngineStyle(tester);

    await typeText(tester, env.controller, 'ok');

    expect(env.controller.sessionController.statusText, contains('Rectangle'));
    expect(env.engine.text, contains('ok'));
    expect(find.byKey(_caret), findsOneWidget);
  });

  testWidgets('I-F11-visible-type-while-ribbon-button-still-focused',
      (tester) async {
    final env = await _pumpRibbonAndCanvas(tester);

    await tester.tap(find.byKey(const Key('insert_text_box')));
    await settleEngineStyle(tester);

    final detectorFinder = find.descendant(
      of: find.byKey(const Key('insert_text_box')),
      matching: find.byType(FocusableActionDetector),
    );
    final node =
        tester.widget<FocusableActionDetector>(detectorFinder.first).focusNode!;
    node.requestFocus();
    await tester.pump();
    expect(node.hasFocus, isTrue);

    await tester.sendKeyEvent(LogicalKeyboardKey.keyH);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyI);
    await env.controller.ensureLayoutReady();
    await tester.pumpAndSettle();

    expect(env.engine.text, contains('hi'));
  });

  testWidgets('I-F11-visible-smartart-hierarchy-then-type-with-ribbon-focus',
      (tester) async {
    final env = await _pumpRibbonAndCanvas(tester);

    await env.controller.insertSmartArt(
      diagramType: EditorController.smartArtHierarchy,
    );
    await settleEngineStyle(tester);

    final detectorFinder = find.descendant(
      of: find.byKey(const Key('insert_smart_art')),
      matching: find.byType(FocusableActionDetector),
    );
    tester
        .widget<FocusableActionDetector>(detectorFinder.first)
        .focusNode!
        .requestFocus();
    await tester.pump();

    await tester.sendKeyEvent(LogicalKeyboardKey.keyA);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyB);
    await settleEngineStyle(tester);

    expect(env.engine.text, contains('ab'));
  });
}

class _Env {
  _Env(this.controller, this.engine);
  final EditorController controller;
  final MockDocumentEngine engine;
}

Future<_Env> _pumpCanvas(
  WidgetTester tester, {
  Uint8List? displayList,
}) async {
  final engine = MockDocumentEngine();
  final controller = createTestEditorController(engine: engine);
  addTearDown(controller.dispose);
  controller.setDisplayListForTest(displayList ?? fakeGlyphDisplayList());

  await tester.binding.setSurfaceSize(const Size(1400, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await pumpTestDocumentView(tester, controller);
  await _focusCanvas(tester);
  return _Env(controller, engine);
}

Future<_Env> _pumpRibbonAndCanvas(WidgetTester tester) async {
  final engine = MockDocumentEngine();
  final controller = createTestEditorController(engine: engine);
  addTearDown(controller.dispose);
  controller.setDisplayListForTest(fakeGlyphDisplayList());

  await tester.binding.setSurfaceSize(const Size(1400, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: Column(
          children: [
            SizedBox(height: 140, child: InsertTab(controller: controller)),
            Expanded(child: DocumentView(controller: controller)),
          ],
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return _Env(controller, engine);
}

Future<void> _focusCanvas(WidgetTester tester) async {
  final surface = find.byType(GlyphEditorSurface).first;
  final box = tester.renderObject(surface) as RenderBox;
  final topLeft = box.localToGlobal(Offset.zero);
  await tester.tapAt(Offset(topLeft.dx + 40, topLeft.dy.clamp(0, 500) + 20));
  await tester.pump();
}

Future<void> _tapShapeInterior(WidgetTester tester) async {
  final surface = find.byType(GlyphEditorSurface).first;
  final box = tester.renderObject(surface) as RenderBox;
  final global = box.localToGlobal(const Offset(100, 100));
  await tester.tapAt(global);
}

Uint8List _shapeDisplayList({
  required String shapeId,
  required Rect rect,
}) {
  final out = BytesBuilder();
  void u32(int value) {
    final data = ByteData(4)..setUint32(0, value, Endian.little);
    out.add(data.buffer.asUint8List());
  }

  void u64(int value) {
    final data = ByteData(8)..setUint64(0, value, Endian.little);
    out.add(data.buffer.asUint8List());
  }

  void f32(double value) {
    final data = ByteData(4)..setFloat32(0, value, Endian.little);
    out.add(data.buffer.asUint8List());
  }

  u32(7); // file version
  u64(1);
  f32(612);
  f32(792);
  u32(0); // glyphs
  u32(0); // rects
  u32(0); // paths
  u32(0); // images
  u32(1); // one shape
  f32(rect.left);
  f32(rect.top);
  f32(rect.width);
  f32(rect.height);
  final idBytes = shapeId.codeUnits;
  u32(idBytes.length);
  out.add(idBytes);
  return out.toBytes();
}
