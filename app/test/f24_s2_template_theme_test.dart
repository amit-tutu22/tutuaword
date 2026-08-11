import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/document_templates.dart';
import 'package:tutuaword/ui/ribbon_tabs/design_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F24.S2 Theme binding', () {
    test('u_f24_s2_template_theme_map all templates bind gallery themes', () {
      expect(DocumentTemplateSpec.all, hasLength(7));
      for (final template in DocumentTemplateSpec.all) {
        expect(
          DocumentTemplateSpec.galleryThemeNames,
          contains(template.themeName),
          reason: '${template.id} theme ${template.themeName}',
        );
      }
      expect(DocumentTemplateSpec.resume.themeName, 'Facet');
      expect(DocumentTemplateSpec.letter.themeName, 'Office');
      expect(DocumentTemplateSpec.invoice.themeName, 'Ion');
    });

    test('I-F24-S2-new-from-resume-theme applies Facet', () async {
      final engine = MockDocumentEngine(initialText: '');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      expect(controller.documentThemeName, 'Office');
      await controller.newFromTemplate(DocumentTemplateSpec.resume);

      expect(controller.documentThemeName, 'Facet');
      expect(
        controller.sessionController.statusText,
        contains('New from template: Resume'),
      );
      expect(controller.sessionController.statusText, contains('Facet'));
      expect(controller.documentText, contains('Professional Resume'));
    });

    testWidgets('I-F24-S2-design-tab-reflects-template', (tester) async {
      final engine = MockDocumentEngine(initialText: '');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.newFromTemplate(DocumentTemplateSpec.invoice);
      expect(controller.documentThemeName, 'Ion');

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(height: 120, child: DesignTab(controller: controller)),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(controller.documentThemeName, 'Ion');
      // Design tab lists all gallery names; selection follows controller.
      expect(find.text('Ion'), findsOneWidget);
    });
  });
}
