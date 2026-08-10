import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/document_templates.dart';
import 'package:tutuaword/ui/new_from_template_dialog.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F24.S1 Built-in template pack', () {
    test('seven starter templates are registered', () {
      expect(DocumentTemplateSpec.all, hasLength(7));
      expect(DocumentTemplateSpec.byId('resume'), isNotNull);
      expect(
        DocumentTemplateSpec.all.map((t) => t.id).toSet(),
        containsAll([
          'resume',
          'letter',
          'invoice',
          'brochure',
          'newsletter',
          'business_proposal',
          'research_paper',
        ]),
      );
    });

    testWidgets('I-F24-S1-new-from-resume opens styled doc untitled', (tester) async {
      final engine = MockDocumentEngine(initialText: '');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                key: const Key('open_templates'),
                onPressed: () => controller.openNewFromTemplateDialog(context),
                child: const Text('Templates'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('open_templates')));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 50));
      expect(find.byKey(const Key('new_from_template_dialog')), findsOneWidget);
      expect(find.byKey(const Key('template_resume')), findsOneWidget);

      await tester.ensureVisible(find.byKey(const Key('template_resume')));
      await tester.tap(find.byKey(const Key('template_resume')));
      await tester.pump();
      expect(find.byKey(const Key('template_gallery_selection')), findsOneWidget);
      await tester.tap(find.byKey(const Key('template_gallery_create')));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 50));

      expect(controller.currentPath, isNull);
      expect(
        controller.documentText,
        contains(DocumentTemplateSpec.resume.markerText),
        reason: 'Resume template body should include styled heading text',
      );
      expect(
        controller.sessionController.statusText,
        contains('New from template: Resume'),
      );
    });

    test('newFromTemplate loads asset bytes into engine', () async {
      final engine = MockDocumentEngine(initialText: 'old');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await controller.newFromTemplate(DocumentTemplateSpec.resume);

      expect(controller.currentPath, isNull);
      expect(controller.documentText, contains('Professional Resume'));
      expect(controller.documentText, contains('Experience'));
    });

    test('resume asset is a real docx zip', () async {
      final data = await rootBundle.load(DocumentTemplateSpec.resume.assetPath);
      final bytes = data.buffer.asUint8List();
      expect(bytes.length, greaterThan(100));
      expect(bytes[0], 0x50); // P
      expect(bytes[1], 0x4B); // K
    });
  });
}
