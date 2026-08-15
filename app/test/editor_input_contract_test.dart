import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/editor_input.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('editor input contract', () {
    test('hello + Enter creates one newline and caret on the new line', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'hello');
      await sendEditorInputDirect(controller, const EditorInputEvent.newline());

      expect(controller.documentText, 'hello\n');
      expect(controller.caretOffset, 6);
    });

    test('hello + Enter + x does not duplicate hello', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'hello');
      await sendEditorInputDirect(controller, const EditorInputEvent.newline());
      await sendEditorInputDirect(controller, EditorInputEvent.character('x'));

      expect(controller.documentText, 'hello\nx');
      expect(controller.documentText, isNot(contains('hellohello')));
      expect(controller.documentText, isNot('xhello'));
    });

    test('mid-line Enter splits suffix to the next line', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'hello');
      await sendEditorInputDirect(controller, const EditorInputEvent.arrowLeft());
      await sendEditorInputDirect(controller, const EditorInputEvent.arrowLeft());
      await sendEditorInputDirect(controller, const EditorInputEvent.newline());

      expect(controller.documentText, 'hel\nlo');
      expect(controller.caretOffset, 4);
    });

    test('Space and Tab insert and advance the caret', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'a');
      await sendEditorInputDirect(controller, EditorInputEvent.character(' '));
      expect(controller.documentText, 'a ');
      expect(controller.caretOffset, 2);

      await sendEditorInputDirect(controller, const EditorInputEvent.tab());
      expect(controller.documentText, 'a \t');
      expect(controller.caretOffset, 3);
    });

    test('Left/Right move without inserting text', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'ab');
      expect(controller.caretOffset, 2);

      await sendEditorInputDirect(controller, const EditorInputEvent.arrowLeft());
      expect(controller.documentText, 'ab');
      expect(controller.caretOffset, 1);

      await sendEditorInputDirect(controller, const EditorInputEvent.arrowRight());
      expect(controller.documentText, 'ab');
      expect(controller.caretOffset, 2);
    });

    test('Left from document start stays at offset 0', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'a');
      await sendEditorInputDirect(controller, const EditorInputEvent.arrowLeft());
      await sendEditorInputDirect(controller, const EditorInputEvent.arrowLeft());

      expect(controller.documentText, 'a');
      expect(controller.caretOffset, 0);
    });

    test('rapid Enter creates five newlines without duplicating text', () async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'seed');
      for (var i = 0; i < 5; i++) {
        await sendEditorInputDirect(controller, const EditorInputEvent.newline());
      }

      expect(controller.documentText, 'seed\n\n\n\n\n');
      expect(controller.caretOffset, controller.documentText.length);
    });
  });
}
