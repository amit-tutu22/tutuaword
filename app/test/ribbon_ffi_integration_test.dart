import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/bridge/native_engine.dart';
import 'package:tutuaword/bridge/native_event_router.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

import 'editor_test_helpers.dart';
import 'native_ffi_test_helpers.dart';

/// Flutter HomeTab callbacks ↔ Rust FFI engine.
///
/// Pointer hit-testing is covered by `ribbon_click_events_test.dart`.
/// This file verifies those same callbacks mutate the native document when
/// `libtw_ffi` is loaded. Soft-skips when the library is absent.
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  tearDown(() {
    NativeEventRouter.instance.reset();
  });

  tearDownAll(() async {
    await NativeEngine.shutdownAsync();
  });

  Map<String, dynamic>? nativeCharFormat(NativeEngine engine) {
    final runId = nativeDefaultRunId(engine);
    if (runId == null) return null;
    final raw = engine.fetchCaretFormat(runId);
    if (raw == null || raw.isEmpty) return null;
    final json = jsonDecode(raw) as Map<String, dynamic>;
    return json['char_format'] as Map<String, dynamic>?;
  }

  testWidgets('I-ribbon-ffi-home-tab-button-wiring', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);

    await pumpWideRibbon(
      tester,
      SizedBox(height: 140, child: HomeTab(controller: controller)),
    );

    // Ensure Bold/Italic/Underline/Center are enabled RibbonToggleButtons.
    RibbonToggleButton bold = tester.widget(
      find.ancestor(
        of: find.byIcon(Icons.format_bold),
        matching: find.byType(RibbonToggleButton),
      ),
    );
    expect(bold.onPressed, isNotNull);
    expect(bold.tooltip, 'Bold');

    RibbonToggleButton italic = tester.widget(
      find.ancestor(
        of: find.byIcon(Icons.format_italic),
        matching: find.byType(RibbonToggleButton),
      ),
    );
    expect(italic.onPressed, isNotNull);

    RibbonToggleButton underline = tester.widget(
      find.ancestor(
        of: find.byIcon(Icons.format_underline),
        matching: find.byType(RibbonToggleButton),
      ),
    );
    expect(underline.onPressed, isNotNull);

    RibbonToggleButton center = tester.widget(
      find.ancestor(
        of: find.byIcon(Icons.format_align_center),
        matching: find.byType(RibbonToggleButton),
      ),
    );
    expect(center.onPressed, isNotNull);
  });

  test('I-ribbon-ffi-engine-connected-on-desktop', () async {
    if (!await nativeFfiEventsAvailable()) return;

    final store = DocumentSessionStore(
      root: Directory.systemTemp.createTempSync('tutuaword_ribbon_ffi_conn_'),
    );
    final controller = EditorController(
      sessionStore: store,
      enableAutosave: false,
    );
    addTearDown(controller.dispose);

    expect(controller.isEngineConnected, isTrue);
    expect(controller.statusText, contains('Rust engine connected'));
  });

  test('I-ribbon-ffi-home-commands-mutate-rust-engine', () async {
    if (!await nativeFfiEventsAvailable()) return;

    final store = DocumentSessionStore(
      root: Directory.systemTemp.createTempSync('tutuaword_ribbon_ffi_api_'),
    );
    final controller = EditorController(
      sessionStore: store,
      enableAutosave: false,
    );
    addTearDown(controller.dispose);
    if (!controller.isEngineConnected) return;

    await typeTextDirect(controller, 'API');
    await controller.ensureLayoutReady();
    expect(controller.documentText, contains('API'));

    // Select typed text so char/para format applies to a non-empty range
    // (collapsed caret alone is a typing attribute, not a run mutation).
    await controller.selectAll();
    await controller.ensureLayoutReady();

    // Same callbacks HomeTab wires on Bold / Italic / Underline / Center.
    controller.toggleBold();
    await controller.ensureLayoutReady();
    expect(controller.bold, isTrue);

    controller.toggleItalic();
    await controller.ensureLayoutReady();
    expect(controller.italic, isTrue);

    controller.toggleUnderline();
    await controller.ensureLayoutReady();
    expect(controller.underline, isTrue);

    controller.setAlignment(TextAlign.center);
    await controller.ensureLayoutReady();
    expect(controller.alignment, TextAlign.center);

    final engine = NativeEngine.load()!;
    final format = nativeCharFormat(engine);
    expect(format?['bold'], isTrue);
    expect(format?['italic'], isTrue);
    expect(format?['underline'], anyOf(isTrue, 'Single', 'single'));

    final runId = nativeDefaultRunId(engine)!;
    final para = (jsonDecode(engine.fetchCaretFormat(runId)!)
        as Map<String, dynamic>)['para_format'] as Map<String, dynamic>;
    expect(para['alignment']?.toString().toLowerCase(), 'center');
  });
}
