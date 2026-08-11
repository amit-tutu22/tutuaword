import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_inspect_finding.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/document_inspector_dialog.dart';

DocumentSessionStore _isolatedStore(String prefix) => DocumentSessionStore(
      root: Directory.systemTemp.createTempSync(prefix),
    );

void main() {
  group('F22.S3 document inspector', () {
    test('I-F22-S3-remove-selected-categories', () async {
      final engine = MockDocumentEngine();
      engine.setDocumentInspectCountsForTest(
        comments: 2,
        title: 'Secret',
        author: 'Ada',
        hiddenText: 1,
      );
      final controller = EditorController(
        engine: engine,
        sessionStore: _isolatedStore('tutuaword_f22_s3_remove_'),
        enableAutosave: false,
      );
      addTearDown(controller.dispose);

      expect(controller.sessionController.fetchDocumentInspectJson(),
          contains('comments'));
      expect(
        controller.sessionController.removeInspectFindings(
          comments: true,
          metadata: true,
          hiddenText: true,
        ),
        isTrue,
      );
      expect(controller.sessionController.fetchDocumentInspectJson(), '[]');
      expect(controller.documentProperties.title, isNull);
      expect(controller.documentProperties.author, isNull);
      expect(controller.statusText, contains('inspected'));
    });

    testWidgets('I-F22-S3-inspect-dialog-remove', (tester) async {
      DocumentInspectRemoval? result;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              return Scaffold(
                body: TextButton(
                  key: const Key('open_inspect'),
                  onPressed: () async {
                    result = await DocumentInspectorDialog.show(
                      context,
                      const [
                        DocumentInspectFinding(
                          category: 'comments',
                          count: 1,
                          message: '1 comment',
                        ),
                        DocumentInspectFinding(
                          category: 'metadata',
                          count: 2,
                          message: '2 document properties',
                        ),
                      ],
                    );
                  },
                  child: const Text('Open'),
                ),
              );
            },
          ),
        ),
      );

      await tester.tap(find.byKey(const Key('open_inspect')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('document_inspector_dialog')), findsOneWidget);

      await tester.tap(find.byKey(const Key('document_inspector_remove')));
      await tester.pumpAndSettle();

      expect(result, isNotNull);
      expect(result!.comments, isTrue);
      expect(result!.metadata, isTrue);
      expect(result!.hiddenText, isFalse);
    });
  });
}
