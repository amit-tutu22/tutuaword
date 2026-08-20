// Word keyboard mapping, exercised inside a real iOS/macOS/Android binary.
//
// Unlike `test/word_keyboard_mapping_test.dart`, this drives the shipping
// widget tree with the FFI engine, the platform text input and the platform
// focus tree, so it catches wiring that only breaks on device — the hidden
// UIKit text field swallowing a chord, or the Rust engine disagreeing with the
// mock about caret arithmetic.
//
// Debug harness only: `integration_test` is a dev dependency and this file sits
// outside `lib/`, so nothing here is compiled into a release build. The guard in
// `main` is a second line of defence if someone wires it up anyway.
//
// Run with: flutter test integration_test/word_keyboard_mapping_test.dart -d <device>

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:tutuaword/bridge/engine_bootstrap.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/editor_screen.dart';

/// Mounts the shipping editor on the real engine and returns its controller.
Future<EditorController> pumpEditor(WidgetTester tester) async {
  final controller = EditorController(enableAutosave: false);
  addTearDown(controller.dispose);
  expect(
    controller.isEngineConnected,
    isTrue,
    reason: 'FFI engine did not load in this binary',
  );

  await tester.pumpWidget(
    MaterialApp(home: EditorScreen(controller: controller)),
  );
  await tester.pumpAndSettle();
  // The FFI engine outlives the widget tree, so each test starts from a blank
  // document instead of inheriting the previous one's text.
  await controller.newDocument();
  await tester.pumpAndSettle();
  await controller.ensureLayoutReady();
  controller.ensureGlyphCaret();
  expect(controller.documentText, isEmpty);
  return controller;
}

/// Pumps until [probe] matches [matcher], for state the engine reports back
/// asynchronously (caret format, layout-derived offsets).
Future<void> expectEventually(
  WidgetTester tester,
  Object? Function() probe,
  Object? matcher, {
  Duration timeout = const Duration(seconds: 3),
}) async {
  final deadline = DateTime.now().add(timeout);
  while (DateTime.now().isBefore(deadline)) {
    if (wrapMatcher(matcher).matches(probe(), <Object?, Object?>{})) return;
    await tester.pump(const Duration(milliseconds: 50));
  }
  expect(probe(), matcher);
}

/// Types [text] the way that platform's keyboard does.
///
/// On iOS and Android the soft-keyboard overlay is a hidden field and the editor
/// reads input as the growth of its value, so the new text is appended to
/// whatever is there; passing [text] alone would shorten the value and be read
/// as backspaces. Desktop has no such field and takes key events instead.
Future<void> typeOnKeyboard(
  WidgetTester tester,
  EditorController controller,
  String text,
) async {
  final field = find.byType(TextField);
  if (field.evaluate().isEmpty) {
    for (final character in text.split('')) {
      await tester.sendKeyEvent(LogicalKeyboardKey(character.codeUnitAt(0)));
    }
  } else {
    final existing = tester.widget<TextField>(field.first).controller?.text ?? '';
    await tester.enterText(field.first, '$existing$text');
  }
  await tester.pumpAndSettle();
  await controller.ensureLayoutReady();
}

/// Presses [key] with the given modifiers held, as a hardware keyboard would.
Future<void> sendChord(
  WidgetTester tester,
  EditorController controller,
  LogicalKeyboardKey key, {
  bool meta = false,
  bool control = false,
  bool shift = false,
  bool alt = false,
}) async {
  if (meta) await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
  if (control) await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
  if (shift) await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
  if (alt) await tester.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
  await tester.sendKeyEvent(key);
  if (alt) await tester.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
  if (shift) await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
  if (control) await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
  if (meta) await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
  await tester.pumpAndSettle();
  await controller.ensureLayoutReady();
}

void main() {
  if (kReleaseMode) return;

  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  setUpAll(() async {
    await warmDocumentEngine();
  });

  testWidgets('the soft keyboard reaches the engine', (tester) async {
    final controller = await pumpEditor(tester);
    await typeOnKeyboard(tester, controller, 'alpha beta');

    expect(controller.documentText, 'alpha beta');
  });

  testWidgets('Cmd+B, Cmd+I and Cmd+U toggle character formatting',
      (tester) async {
    final controller = await pumpEditor(tester);
    await typeOnKeyboard(tester, controller, 'alpha');

    await sendChord(tester, controller, LogicalKeyboardKey.keyB, meta: true);
    await expectEventually(tester, () => controller.bold, isTrue);

    await sendChord(tester, controller, LogicalKeyboardKey.keyI, meta: true);
    await expectEventually(tester, () => controller.italic, isTrue);

    await sendChord(tester, controller, LogicalKeyboardKey.keyU, meta: true);
    await expectEventually(tester, () => controller.underline, isTrue);

    // The chord must not leak its letter into the document.
    expect(controller.documentText, 'alpha');
  });

  testWidgets('Cmd+Z undoes and Cmd+Shift+Z redoes', (tester) async {
    final controller = await pumpEditor(tester);
    await typeOnKeyboard(tester, controller, 'abc');

    await sendChord(tester, controller, LogicalKeyboardKey.keyZ, meta: true);
    await expectEventually(tester, () => controller.documentText, isNot('abc'));

    await sendChord(
      tester,
      controller,
      LogicalKeyboardKey.keyZ,
      meta: true,
      shift: true,
    );
    await expectEventually(tester, () => controller.documentText, 'abc');
  });

  testWidgets('Shift+Enter breaks the line inside the paragraph',
      (tester) async {
    final controller = await pumpEditor(tester);
    await typeOnKeyboard(tester, controller, 'first');
    final runBefore = controller.caretRunId;

    await sendChord(tester, controller, LogicalKeyboardKey.enter, shift: true);

    expect(controller.documentText, contains('\n'));
    expect(
      controller.caretRunId,
      runBefore,
      reason: 'a manual break must stay in the run Enter would have split',
    );
  });

  testWidgets('Cmd+Left and Cmd+Right land on the line edges', (tester) async {
    // An iPad Magic Keyboard has no Home/End, so Word uses Cmd+Arrow there.
    final controller = await pumpEditor(tester);
    await typeOnKeyboard(tester, controller, 'alpha beta');

    await sendChord(tester, controller, LogicalKeyboardKey.arrowLeft,
        meta: true);
    await expectEventually(tester, () => controller.caretOffset, 0);

    await sendChord(tester, controller, LogicalKeyboardKey.arrowRight,
        meta: true);
    await expectEventually(
        tester, () => controller.caretOffset, 'alpha beta'.length);
  });

  testWidgets('Option+Arrow moves by word', (tester) async {
    final controller = await pumpEditor(tester);
    await typeOnKeyboard(tester, controller, 'alpha beta gamma');

    await sendChord(tester, controller, LogicalKeyboardKey.arrowLeft,
        alt: true);
    await expectEventually(
        tester, () => controller.caretOffset, 'alpha beta '.length);

    await sendChord(tester, controller, LogicalKeyboardKey.arrowRight,
        alt: true);
    await expectEventually(
        tester, () => controller.caretOffset, 'alpha beta gamma'.length);
  });

  testWidgets('Option+Backspace deletes the previous word', (tester) async {
    final controller = await pumpEditor(tester);
    await typeOnKeyboard(tester, controller, 'alpha beta');

    await sendChord(tester, controller, LogicalKeyboardKey.backspace,
        alt: true);

    await expectEventually(tester, () => controller.documentText, 'alpha ');
  });

  testWidgets('paragraph chords reach the paragraph', (tester) async {
    final controller = await pumpEditor(tester);
    await typeOnKeyboard(tester, controller, 'alpha');

    await sendChord(tester, controller, LogicalKeyboardKey.keyE, meta: true);
    await expectEventually(
        tester, () => controller.alignment, TextAlign.center);

    await sendChord(tester, controller, LogicalKeyboardKey.keyJ, meta: true);
    await expectEventually(
        tester, () => controller.alignment, TextAlign.justify);
  });

  testWidgets('Ctrl+Up and Ctrl+Down move by paragraph', (tester) async {
    // The real engine keeps each paragraph in its own run, so this is the case
    // the mock cannot represent: the caret must cross a run, not an offset.
    final controller = await pumpEditor(tester);
    await typeOnKeyboard(tester, controller, 'alpha');
    await controller.insertGlyphParagraphBreak();
    await typeOnKeyboard(tester, controller, 'beta');
    final secondRun = controller.caretRunId;

    await sendChord(tester, controller, LogicalKeyboardKey.arrowUp,
        control: true);
    await expectEventually(tester, () => controller.caretOffset, 0);
    expect(controller.caretRunId, secondRun);

    await sendChord(tester, controller, LogicalKeyboardKey.arrowUp,
        control: true);
    await expectEventually(tester, () => controller.caretRunId,
        isNot(secondRun));
    expect(controller.caretOffset, 0);

    await sendChord(tester, controller, LogicalKeyboardKey.arrowDown,
        control: true);
    await expectEventually(tester, () => controller.caretRunId, secondRun);
  });

  testWidgets('Alt+Shift+Up moves the paragraph', (tester) async {
    final controller = await pumpEditor(tester);
    await typeOnKeyboard(tester, controller, 'alpha');
    await controller.insertGlyphParagraphBreak();
    await typeOnKeyboard(tester, controller, 'beta');

    await sendChord(
      tester,
      controller,
      LogicalKeyboardKey.arrowUp,
      shift: true,
      alt: true,
    );

    await expectEventually(
      tester,
      () => controller.documentText,
      contains('beta\nalpha'),
    );
  });

  testWidgets('Cmd+T hangs the first line and Cmd+Q clears the paragraph',
      (tester) async {
    final controller = await pumpEditor(tester);
    await typeOnKeyboard(tester, controller, 'alpha');

    await sendChord(tester, controller, LogicalKeyboardKey.keyT, meta: true);
    await expectEventually(tester, () => controller.indentLeft, 36.0);
    expect(controller.indentFirstLine, -36.0);

    await sendChord(tester, controller, LogicalKeyboardKey.digit0, meta: true);
    await expectEventually(tester, () => controller.spaceBefore, 12.0);

    await sendChord(tester, controller, LogicalKeyboardKey.keyQ, meta: true);
    await expectEventually(tester, () => controller.indentLeft, 0.0);
    expect(controller.spaceBefore, 0.0);
  });

  testWidgets('Cmd+Shift+E turns track changes on', (tester) async {
    final controller = await pumpEditor(tester);

    expect(controller.trackChanges, isFalse);
    await sendChord(
      tester,
      controller,
      LogicalKeyboardKey.keyE,
      meta: true,
      shift: true,
    );
    expect(controller.trackChanges, isTrue);
  });
}
