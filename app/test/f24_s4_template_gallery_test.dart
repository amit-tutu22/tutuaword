import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/document_templates.dart';
import 'package:tutuaword/ui/new_from_template_dialog.dart';
import 'package:tutuaword/ui/word_theme.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F24.S4 Template gallery UI', () {
    test('U-F24-S4-filter-query', () {
      final all = TemplateGalleryFilter.filterBuiltins(
        query: '',
        category: TemplateGalleryCategory.all,
      );
      expect(all, hasLength(7));

      final resume = TemplateGalleryFilter.filterBuiltins(
        query: 'resume',
        category: TemplateGalleryCategory.all,
      );
      expect(resume.map((t) => t.id), ['resume']);

      final none = TemplateGalleryFilter.filterBuiltins(
        query: 'zzzz',
        category: TemplateGalleryCategory.all,
      );
      expect(none, isEmpty);
    });

    test('U-F24-S4-filter-category', () {
      final user = [
        UserTemplateEntry(
          id: 'mine-1',
          title: 'Mine',
          themeName: 'Office',
          fileName: 'mine-1.docx',
          createdAt: DateTime.utc(2026, 1, 1),
        ),
      ];
      expect(
        TemplateGalleryFilter.filterBuiltins(
          query: '',
          category: TemplateGalleryCategory.mine,
        ),
        isEmpty,
      );
      expect(
        TemplateGalleryFilter.filterUser(
          userTemplates: user,
          query: '',
          category: TemplateGalleryCategory.builtin,
        ),
        isEmpty,
      );
      expect(
        TemplateGalleryFilter.filterUser(
          userTemplates: user,
          query: '',
          category: TemplateGalleryCategory.mine,
        ),
        hasLength(1),
      );
    });

    test('U-F24-S4-preview-kinds', () {
      for (final template in DocumentTemplateSpec.all) {
        expect(
          TemplatePreviewKind.forBuiltin(template.id),
          isNot(TemplatePreviewKind.user),
          reason: template.id,
        );
      }
      expect(templateThemeAccent('Facet'), const Color(0xFF217346));
      expect(templateThemeAccent('Ion'), const Color(0xFFC45911));
      expect(templateThemeAccent('Office'), WordTheme.activeTabUnderline);
    });

    testWidgets('I-F24-S4-gallery-shows-cards', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: ElevatedButton(
                onPressed: () => NewFromTemplateDialog.show(context),
                child: const Text('Open'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open'));
      await tester.pump();

      expect(find.byKey(const Key('new_from_template_dialog')), findsOneWidget);
      expect(find.byKey(const Key('template_gallery_search')), findsOneWidget);
      expect(find.byKey(const Key('builtin_templates_section')), findsOneWidget);
      for (final template in DocumentTemplateSpec.all) {
        expect(find.byKey(Key('template_${template.id}')), findsOneWidget);
      }
    });

    testWidgets('I-F24-S4-gallery-filter', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: ElevatedButton(
                onPressed: () => NewFromTemplateDialog.show(context),
                child: const Text('Open'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open'));
      await tester.pump();

      await tester.enterText(
        find.byKey(const Key('template_gallery_search')),
        'invoice',
      );
      await tester.pump();

      expect(find.byKey(const Key('template_invoice')), findsOneWidget);
      expect(find.byKey(const Key('template_resume')), findsNothing);
    });

    testWidgets('I-F24-S4-gallery-create', (tester) async {
      final engine = MockDocumentEngine(initialText: '');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: ElevatedButton(
                key: const Key('open_gallery'),
                onPressed: () => controller.openNewFromTemplateDialog(context),
                child: const Text('Open'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('open_gallery')));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 50));
      await tester.ensureVisible(find.byKey(const Key('template_letter')));
      await tester.tap(find.byKey(const Key('template_letter')));
      await tester.pump();
      expect(find.byKey(const Key('template_gallery_selection')), findsOneWidget);

      await tester.tap(find.byKey(const Key('template_gallery_create')));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 50));

      expect(controller.currentPath, isNull);
      expect(
        controller.documentText,
        contains(DocumentTemplateSpec.letter.markerText),
      );
      expect(controller.documentThemeName, 'Office');
    });

    testWidgets('I-F24-S4-gallery-my-templates-category', (tester) async {
      final user = [
        UserTemplateEntry(
          id: 'custom-1',
          title: 'Custom One',
          themeName: 'Facet',
          fileName: 'custom-1.docx',
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

      expect(find.byKey(const Key('my_templates_section')), findsOneWidget);
      expect(find.byKey(const Key('user_template_custom-1')), findsOneWidget);

      await tester.tap(find.byKey(const Key('template_gallery_cat_mine')));
      await tester.pump();
      expect(find.byKey(const Key('user_template_custom-1')), findsOneWidget);
      expect(find.byKey(const Key('template_resume')), findsNothing);
    });
  });
}
