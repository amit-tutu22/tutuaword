import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F05.S2 Multi-level lists', () {
    /// I-F05-S2-promote-demote: Tab at the start of a list item increases ilvl.
    /// Word only changes the level from the start of the paragraph.
    testWidgets('I-F05-S2-promote-demote Tab increases list level', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeTextDirect(controller, 'Item');
      controller.applyBulletList();
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.isInList, isTrue);
      expect(controller.listLevel, 0);
      expect(mockEngineNumbering(engine)?['level'], 0);

      controller.moveGlyphCaretToLineEdge(toEnd: false);
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.listLevel, 1);
      expect(mockEngineNumbering(engine)?['level'], 1);
      expect(mockEngineNumbering(engine)?['numbering_id'], 1);
      expect(controller.documentText, isNot(contains('\t')));
    });

    testWidgets('Tab mid-item inserts a tab like Word', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeTextDirect(controller, 'Item');
      controller.applyBulletList();
      await controller.ensureLayoutReady();
      expect(controller.caretOffset, greaterThan(0));

      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.listLevel, 0);
      expect(controller.documentText, contains('\t'));
    });

    testWidgets('Shift+Tab demotes list level from anywhere in the item',
        (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeTextDirect(controller, 'Item');
      controller.applyBulletList();
      await controller.ensureLayoutReady();

      controller.moveGlyphCaretToLineEdge(toEnd: false);
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await controller.ensureLayoutReady();
      expect(controller.listLevel, 1);

      await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.listLevel, 0);
      expect(mockEngineNumbering(engine)?['level'], 0);
    });

    testWidgets('Tab on non-list paragraph inserts tab character', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeTextDirect(controller, 'Plain');
      expect(controller.isInList, isFalse);

      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.documentText, contains('\t'));
      expect(mockEngineNumbering(engine), isNull);
    });

    testWidgets('promote at max level is a no-op', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Item');
      controller.applyBulletList();
      await controller.ensureLayoutReady();

      controller.increaseIndent();
      await controller.ensureLayoutReady();
      expect(controller.listLevel, 1);

      controller.increaseIndent();
      await controller.ensureLayoutReady();

      expect(controller.listLevel, 1);
      expect(mockEngineNumbering(engine)?['level'], 1);
    });

    testWidgets('numbered promote syncs outline level', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Section');
      controller.applyNumberedList();
      await controller.ensureLayoutReady();
      expect(mockEngineParaFormat(engine)['outline_level'], 0);

      controller.increaseIndent();
      await controller.ensureLayoutReady();

      expect(controller.listLevel, 1);
      expect(mockEngineParaFormat(engine)['outline_level'], 1);
    });
  });
}
