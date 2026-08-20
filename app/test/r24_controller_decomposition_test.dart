import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/doc_range.dart';
import 'package:tutuaword/editor/editor_controller.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('R2.4 EditorController decomposition', () {
    test('sub-controllers exist and editor_controller.dart stays under budget', () {
      final controller = EditorController(engine: MockDocumentEngine());
      addTearDown(controller.dispose);

      expect(controller.view, isNotNull);
      expect(controller.selectionController, isNotNull);
      expect(controller.formattingController, isNotNull);
      expect(controller.sessionController, isNotNull);

      final source = File('lib/editor/editor_controller.dart');
      expect(source.existsSync(), isTrue);
      final lineCount = source.readAsLinesSync().length;
      // Facade still owns feature entrypoints; keep growth bounded.
      expect(lineCount, lessThan(5000), reason: 'editor_controller.dart has $lineCount lines');
    });

    test('selection uses DocRange model', () {
      final controller = EditorController(engine: MockDocumentEngine());
      addTearDown(controller.dispose);

      expect(controller.selection, isA<DocRange?>());
      expect(controller.hasGlyphSelection, isFalse);

      controller.hitTestAt(0, 72, 87);
      expect(controller.selection, isNotNull);
      expect(controller.selection!.isCollapsed, isTrue);
    });

    test('mock engine supports glyph editing without TextField fallback', () async {
      final engine = MockDocumentEngine();
      final controller = EditorController(engine: engine);
      addTearDown(controller.dispose);

      expect(controller.preferTextRendering, isFalse);
      expect(controller.usesGlyphRendering, isTrue);
      expect(controller.textController, isNull);

      controller.hitTestAt(0, 72, 87);
      await controller.insertGlyphCharacter('H');
      expect(engine.text, 'H');
    });

    test('clipboard cut/copy uses DocRange via engine', () async {
      final engine = MockDocumentEngine(initialText: 'Hello');
      final controller = EditorController(engine: engine);
      addTearDown(controller.dispose);

      controller.hitTestAt(0, 72, 87);
      await controller.selectAll();
      expect(controller.selectedText, 'Hello');
      expect(controller.canCutOrCopy, isTrue);
    });
  });
}
