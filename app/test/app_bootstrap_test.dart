import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/app_bootstrap.dart';
import 'package:tutuaword/editor/editor_screen.dart';

void main() {
  testWidgets('U-bootstrap-engine-failure-blocks-editor', (tester) async {
    var attempts = 0;
    await tester.pumpWidget(
      MaterialApp(
        home: AppBootstrap(
          warmEngine: () async {
            attempts += 1;
            throw StateError('libtw_ffi missing');
          },
          editorBuilder: (_) => const SizedBox(key: Key('fake_editor')),
        ),
      ),
    );

    // Post-frame warm + settle error UI.
    await tester.pump();
    await tester.pump();

    expect(find.byType(EditorScreen), findsNothing);
    expect(find.byKey(const Key('fake_editor')), findsNothing);
    expect(find.byKey(const Key('engine_bootstrap_error')), findsOneWidget);
    expect(find.textContaining('libtw_ffi missing'), findsOneWidget);
    expect(attempts, 1);

    await tester.tap(find.byKey(const Key('engine_bootstrap_retry')));
    await tester.pump();
    await tester.pump();

    expect(attempts, 2);
    expect(find.byKey(const Key('fake_editor')), findsNothing);
  });

  testWidgets('U-bootstrap-engine-ready-mounts-editor', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: AppBootstrap(
          warmEngine: () async {},
          editorBuilder: (_) => const SizedBox(key: Key('fake_editor')),
        ),
      ),
    );

    await tester.pump();
    await tester.pump();

    expect(find.byKey(const Key('fake_editor')), findsOneWidget);
    expect(find.byKey(const Key('engine_bootstrap_error')), findsNothing);
  });
}
