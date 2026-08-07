import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  /// Underline value the engine holds for the caret run.
  String engineUnderline(MockDocumentEngine engine) {
    final json = jsonDecode(engine.fetchCaretFormat(engine.defaultRunId)!)
        as Map<String, dynamic>;
    return (json['char_format'] as Map<String, dynamic>)['underline'] as String;
  }

  group('F03.S1 underline', () {
    test('I-F03-S1-underline-toggle applies underline to typed text', () async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'abc');
      expect(controller.documentText, 'abc');

      controller.toggleUnderline();
      await controller.ensureLayoutReady();

      expect(controller.underline, isTrue, reason: 'ribbon should show underline on');
      expect(engineUnderline(engine), 'Single');
    });

    test('I-F03-S1-underline-typing-attribute applies to subsequently typed text', () async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.ensureGlyphCaret();
      controller.toggleUnderline();
      await controller.ensureLayoutReady();

      await typeTextDirect(controller, 'abc');

      expect(controller.documentText, 'abc');
      expect(controller.underline, isTrue, reason: 'ribbon should reflect typing attribute');
      expect(engineUnderline(engine), 'Single');
    });

    test('I-F03-S1-underline-selection applies to select-all range', () async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'hello');

      await controller.selectAll();
      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText, 'hello');

      controller.toggleUnderline();
      await controller.ensureLayoutReady();

      expect(controller.underline, isTrue, reason: 'ribbon should show underline on');
      expect(engineUnderline(engine), 'Single');

      controller.toggleUnderline();
      await controller.ensureLayoutReady();
      expect(engineUnderline(engine), 'None');
    });
  });
}
