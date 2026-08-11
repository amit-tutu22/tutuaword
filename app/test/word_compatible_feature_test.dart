import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/editor/editor_controller.dart';

import 'editor_test_helpers.dart';
import 'native_ffi_test_helpers.dart';

/// I-word-compat: open the Word feature fixture via native FFI and smoke-check
/// render + find + edit on the host platform.
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('I-word-compat open render find edit on native engine', () async {
    if (!await nativeFfiEventsAvailable()) return;

    final store = DocumentSessionStore(
      root: Directory.systemTemp.createTempSync('tutuaword_word_compat_'),
    );
    final controller = EditorController(
      sessionStore: store,
      enableAutosave: false,
    );
    addTearDown(controller.dispose);
    if (!controller.isEngineConnected) return;

    final candidates = [
      File('test/fixtures/word_compatible_feature_test.docx'),
      File('../crates/tw-docx/tests/corpus/word_compatible_feature_test.docx'),
    ];
    final fixture = candidates.firstWhere(
      (f) => f.existsSync(),
      orElse: () => File(''),
    );
    if (!fixture.existsSync()) {
      fail('word_compatible_feature_test.docx fixture missing');
    }

    await controller.openDocumentFromPath(fixture.path);
    await controller.ensureLayoutReady();

    expect(controller.displayListForPage(0), isNotEmpty);
    expect(controller.pageCount, greaterThanOrEqualTo(2));

    final text = controller.documentText;
    expect(text, contains('Microsoft Word-Compatible Feature Test Document'));
    expect(text, contains('PROJECT-ALPHA-2026'));
    expect(text, contains('The quick brown fox jumps over the lazy dog'));
    expect(text, contains('10. Images'));
    expect(text, contains('14. Equations'));
    expect(text, contains('Before '));
    expect(text, contains('[math]'));
    expect(text, contains('Item'));
    expect(text, contains('Quarter'));

    controller.openFindPane();
    controller.setFindQuery('PROJECT-ALPHA-2026');
    if (!controller.findMatchCase) {
      controller.toggleFindMatchCase();
    }
    expect(controller.findStatusText, contains('of 1'));

    controller.ensureGlyphCaret();
    await typeTextDirect(controller, '[FLUTTER-PROBE]');
    await controller.ensureLayoutReady();
    expect(controller.documentText, contains('[FLUTTER-PROBE]'));
  });
}
