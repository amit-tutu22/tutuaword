import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/style_inspector_pane.dart';

import 'editor_test_helpers.dart';

void main() {
  group('F06.S4 Style inspector', () {
    testWidgets('I-F06-S4-inspector-shows-source toggle shows Heading 1 + Bold direct', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'Section title');
      controller.toggleBold();
      controller.applyHeading1();
      await controller.ensureLayoutReady();

      expect(controller.styleInspectorSummary, 'Heading 1 + Bold direct');
      expect(controller.showStyleInspector, isFalse);

      controller.toggleStyleInspector();
      expect(controller.showStyleInspector, isTrue);

      await pumpTestDocumentView(tester, controller);
      expect(find.byType(StyleInspectorPane), findsOneWidget);
      expect(find.text('Heading 1 + Bold direct'), findsOneWidget);
    });
  });
}
