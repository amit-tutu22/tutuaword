import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/editor_controller.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  Future<EditorController> freshController() async {
    final controller = EditorController(enableAutosave: false);
    addTearDown(controller.dispose);
    if (controller.isEngineConnected) {
      await controller.newDocument();
    }
    return controller;
  }

  group('F03.S1 underline', () {
    test('I-F03-S1-underline-toggle applies underline to typed text', () async {
      final controller = await freshController();
      if (!controller.isEngineConnected) return;

      controller.ensureGlyphCaret();
      for (final ch in 'abc'.split('')) {
        controller.insertGlyphCharacter(ch);
      }
      expect(controller.documentText, 'abc');

      controller.toggleUnderline();
      expect(controller.underline, isTrue, reason: 'ribbon should show underline on');

      final snapshot = DisplayListSnapshot.fromBytes(controller.displayListBytes);
      expect(
        snapshot.rectBatch.isNotEmpty,
        isTrue,
        reason: 'underline should emit decoration rects in display list',
      );
    });

    test('I-F03-S1-underline-typing-attribute applies to subsequently typed text', () async {
      final controller = await freshController();
      if (!controller.isEngineConnected) return;

      controller.ensureGlyphCaret();
      controller.toggleUnderline();

      for (final ch in 'abc'.split('')) {
        controller.insertGlyphCharacter(ch);
      }

      final snapshot = DisplayListSnapshot.fromBytes(controller.displayListBytes);
      expect(
        snapshot.rectBatch.isNotEmpty,
        isTrue,
        reason: 'typing with underline on should paint decoration rects',
      );
      expect(controller.underline, isTrue, reason: 'ribbon should reflect typing attribute');
    });

    test('I-F03-S1-underline-selection applies to select-all range', () async {
      final controller = await freshController();
      if (!controller.isEngineConnected) return;

      controller.ensureGlyphCaret();
      for (final ch in 'hello'.split('')) {
        controller.insertGlyphCharacter(ch);
      }

      controller.selectAll();
      expect(controller.hasGlyphSelection, isTrue);

      controller.toggleUnderline();

      final snapshot = DisplayListSnapshot.fromBytes(controller.displayListBytes);
      expect(
        snapshot.rectBatch.isNotEmpty,
        isTrue,
        reason: 'selection underline should produce decoration rects',
      );
      expect(controller.underline, isTrue, reason: 'ribbon should show underline on');
    });
  });
}
