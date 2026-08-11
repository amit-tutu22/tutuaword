import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/document_templates.dart';
import 'package:tutuaword/ui/new_from_template_dialog.dart';
import 'package:tutuaword/ui/save_as_template_dialog.dart';

import 'editor_test_helpers.dart';

DocumentSessionStore _isolatedStore(String prefix) {
  return DocumentSessionStore(root: Directory.systemTemp.createTempSync(prefix));
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F24.S3 Save as template', () {
    test('u_f24_s3_slugify and unique ids', () {
      expect(DocumentTemplateSpec.slugifyTitle('My Resume!'), 'my-resume');
      expect(DocumentTemplateSpec.slugifyTitle('  '), 'template');
      expect(DocumentTemplateSpec.slugifyTitle('A---B'), 'a-b');
    });

    test('U-F24-S3-template-store-roundtrip', () async {
      final store = _isolatedStore('tutuaword_f24_s3_rt_');
      final bytes = Uint8List.fromList('{"body":"Saved body"}'.codeUnits);

      final entry = await store.saveUserTemplate(
        title: 'Client Letter',
        themeName: 'Facet',
        bytes: bytes,
      );

      expect(entry.id, 'client-letter');
      expect(entry.themeName, 'Facet');
      final listed = store.loadUserTemplates();
      expect(listed, hasLength(1));
      expect(listed.first.title, 'Client Letter');

      final read = await store.readUserTemplateBytes(entry.id);
      expect(read, isNotNull);
      expect(String.fromCharCodes(read!), contains('Saved body'));
    });

    test('U-F24-S3-slug-unique', () async {
      final store = _isolatedStore('tutuaword_f24_s3_slug_');
      final a = await store.saveUserTemplate(
        title: 'Report',
        themeName: 'Office',
        bytes: Uint8List.fromList([1]),
      );
      final b = await store.saveUserTemplate(
        title: 'Report',
        themeName: 'Ion',
        bytes: Uint8List.fromList([2]),
      );
      expect(a.id, 'report');
      expect(b.id, 'report-2');
      expect(store.loadUserTemplates().map((e) => e.id).toSet(), {
        'report',
        'report-2',
      });
    });

    test('I-F24-S3-save-as-template-menu', () async {
      final store = _isolatedStore('tutuaword_f24_s3_save_');
      final engine = MockDocumentEngine(initialText: 'Draft for template');
      final controller = createTestEditorController(
        engine: engine,
        sessionStore: store,
      );
      addTearDown(controller.dispose);

      controller.applyDocumentTheme('Ion');
      await Future<void>.delayed(Duration.zero);
      expect(controller.documentThemeName, 'Ion');

      final priorPath = controller.currentPath;
      final entry = await controller.sessionController.saveAsTemplate(
        title: 'Weekly Status',
        themeName: controller.documentThemeName,
      );

      expect(entry, isNotNull);
      expect(entry!.title, 'Weekly Status');
      expect(entry.themeName, 'Ion');
      expect(entry.id, 'weekly-status');
      expect(store.loadUserTemplates(), hasLength(1));
      expect(
        controller.sessionController.statusText,
        contains('Saved template: Weekly Status'),
      );
      expect(controller.currentPath, priorPath);
    });

    testWidgets('I-F24-S3-save-as-template-dialog', (tester) async {
      String? result;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: ElevatedButton(
                onPressed: () async {
                  result = await SaveAsTemplateDialog.show(
                    context,
                    initialTitle: 'My Template',
                  );
                },
                child: const Text('Save Template'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Save Template'));
      await tester.pump();
      expect(find.byKey(const Key('save_as_template_dialog')), findsOneWidget);

      await tester.enterText(
        find.byKey(const Key('save_as_template_name')),
        'Weekly Status',
      );
      await tester.tap(find.byKey(const Key('save_as_template_save')));
      await tester.pump();

      expect(result, 'Weekly Status');
      expect(find.byKey(const Key('save_as_template_dialog')), findsNothing);
    });

    test('I-F24-S3-new-from-user-template', () async {
      final store = _isolatedStore('tutuaword_f24_s3_open_');
      await store.saveUserTemplate(
        title: 'My Invoice',
        themeName: 'Ion',
        bytes: Uint8List.fromList(
          '{"body":"Custom invoice body marker"}'.codeUnits,
        ),
      );

      final engine = MockDocumentEngine(initialText: 'old');
      final controller = createTestEditorController(
        engine: engine,
        sessionStore: store,
      );
      addTearDown(controller.dispose);

      final entry = store.loadUserTemplates().single;
      await controller.newFromUserTemplate(entry);

      expect(controller.currentPath, isNull);
      expect(controller.documentText, contains('Custom invoice body marker'));
      expect(controller.documentThemeName, 'Ion');
      expect(
        controller.sessionController.statusText,
        contains('New from template: My Invoice'),
      );
    });

    testWidgets('new from template lists My Templates section', (tester) async {
      final user = [
        UserTemplateEntry(
          id: 'my-invoice',
          title: 'My Invoice',
          themeName: 'Ion',
          fileName: 'my-invoice.docx',
          createdAt: DateTime.utc(2026, 1, 1),
        ),
      ];
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: ElevatedButton(
                onPressed: () => NewFromTemplateDialog.show(
                  context,
                  userTemplates: user,
                ),
                child: const Text('Open'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open'));
      await tester.pump();
      expect(find.byKey(const Key('template_resume')), findsOneWidget);
      expect(find.byKey(const Key('my_templates_section')), findsOneWidget);
      expect(find.byKey(const Key('user_template_my-invoice')), findsOneWidget);
    });
  });
}
