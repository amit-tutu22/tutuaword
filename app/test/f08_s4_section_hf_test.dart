import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F08.S4 Section-specific headers', () {
    testWidgets('I-F08-S4-linked-section-inherits-header-text', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.openHeaderEdit();
      await settleEngineStyle(tester);
      await controller.insertGlyphCharacter('S');
      await controller.insertGlyphCharacter('h');
      await controller.insertGlyphCharacter('a');
      await settleEngineStyle(tester);
      expect(engine.headerText, 'Sha');

      controller.insertSectionBreak();
      await settleEngineStyle(tester);
      expect(engine.sectionCount, 2);

      controller.selectionController.setCaret(engine.defaultRunId, 0, page: 1);
      await controller.openHeaderEdit();
      await settleEngineStyle(tester);

      expect(controller.headerFooterLinked, isTrue);
      expect(engine.resolvedHeaderText(1), 'Sha');
    });

    testWidgets('I-F08-S4-unlink-allows-distinct-section-header', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.openHeaderEdit();
      await controller.insertGlyphCharacter('A');
      await settleEngineStyle(tester);

      controller.insertSectionBreak();
      await settleEngineStyle(tester);

      controller.selectionController.setCaret(engine.defaultRunId, 0, page: 1);
      await controller.openHeaderEdit();
      await settleEngineStyle(tester);
      await controller.setHeaderFooterLinked(false);
      await settleEngineStyle(tester);

      await controller.insertGlyphCharacter('B');
      await settleEngineStyle(tester);

      expect(engine.resolvedHeaderText(0), 'A');
      expect(engine.resolvedHeaderText(1), contains('B'));
      expect(controller.headerFooterLinked, isFalse);
    });

    testWidgets('I-F08-S4-relink-restores-shared-header', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.openHeaderEdit();
      await controller.insertGlyphCharacter('Z');
      await settleEngineStyle(tester);
      controller.insertSectionBreak();
      await settleEngineStyle(tester);

      controller.selectionController.setCaret(engine.defaultRunId, 0, page: 1);
      await controller.openHeaderEdit();
      await controller.setHeaderFooterLinked(false);
      await controller.insertGlyphCharacter('!');
      await settleEngineStyle(tester);
      expect(engine.resolvedHeaderText(1), '!');

      await controller.setHeaderFooterLinked(true);
      await settleEngineStyle(tester);
      expect(controller.headerFooterLinked, isTrue);
      expect(engine.resolvedHeaderText(1), 'Z');
    });
  });
}
