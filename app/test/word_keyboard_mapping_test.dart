import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/editor_input.dart';
import 'package:tutuaword/editor/editor_screen.dart';
import 'package:tutuaword/editor/web_glyph_text_input.dart';
import 'package:tutuaword/ui/keyboard_shortcuts.dart';

import 'editor_test_helpers.dart';

EditorInputKind? _kindFor(
  LogicalKeyboardKey key, {
  bool shift = false,
  bool control = false,
  bool alt = false,
  bool meta = false,
}) =>
    EditorInputEvent.fromLogicalKey(
      key,
      shift: shift,
      control: control,
      alt: alt,
      meta: meta,
    )?.kind;

/// Presses [key] with Ctrl held (the Windows/Linux convention this app also
/// registers on macOS).
Future<void> _sendCtrlChord(
  WidgetTester tester,
  LogicalKeyboardKey key, {
  bool shift = false,
  bool alt = false,
}) async {
  await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
  if (shift) await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
  if (alt) await tester.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
  await tester.sendKeyEvent(key);
  if (alt) await tester.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
  if (shift) await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
  await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
  await tester.pumpAndSettle();
}

/// Presses [key] with Cmd held — the modifier macOS and iPadOS actually send.
Future<void> _sendMetaChord(
  WidgetTester tester,
  LogicalKeyboardKey key, {
  bool shift = false,
  bool alt = false,
}) async {
  await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
  if (shift) await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
  if (alt) await tester.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
  await tester.sendKeyEvent(key);
  if (alt) await tester.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
  if (shift) await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
  await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
  await tester.pumpAndSettle();
}

/// Runs [body] as if the binary were the macOS or iOS build. The override has
/// to be cleared inside the test body — the framework checks it on the way out.
Future<void> _onPlatform(
  TargetPlatform platform,
  Future<void> Function() body,
) async {
  debugDefaultTargetPlatformOverride = platform;
  try {
    await body();
  } finally {
    debugDefaultTargetPlatformOverride = null;
  }
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('Word chord → editing intent', () {
    test('Enter variants split, break, or paginate', () {
      expect(_kindFor(LogicalKeyboardKey.enter), EditorInputKind.newline);
      expect(
        _kindFor(LogicalKeyboardKey.enter, shift: true),
        EditorInputKind.lineBreak,
      );
      expect(
        _kindFor(LogicalKeyboardKey.enter, control: true),
        EditorInputKind.pageBreak,
      );
    });

    test('Home and End reach the line, Ctrl reaches the document', () {
      expect(_kindFor(LogicalKeyboardKey.home), EditorInputKind.lineStart);
      expect(_kindFor(LogicalKeyboardKey.end), EditorInputKind.lineEnd);
      expect(
        _kindFor(LogicalKeyboardKey.home, control: true),
        EditorInputKind.documentStart,
      );
      expect(
        _kindFor(LogicalKeyboardKey.end, control: true),
        EditorInputKind.documentEnd,
      );
    });

    test('both Word conventions for word and document motion', () {
      expect(
        _kindFor(LogicalKeyboardKey.arrowLeft, control: true),
        EditorInputKind.wordLeft,
      );
      expect(
        _kindFor(LogicalKeyboardKey.arrowRight, alt: true),
        EditorInputKind.wordRight,
      );
      expect(
        _kindFor(LogicalKeyboardKey.arrowLeft, meta: true),
        EditorInputKind.lineStart,
      );
      expect(
        _kindFor(LogicalKeyboardKey.arrowUp, meta: true),
        EditorInputKind.documentStart,
      );
      expect(
        _kindFor(LogicalKeyboardKey.arrowDown, meta: true),
        EditorInputKind.documentEnd,
      );
    });

    test('Page Up and Page Down are claimed', () {
      expect(_kindFor(LogicalKeyboardKey.pageUp), EditorInputKind.pageUp);
      expect(_kindFor(LogicalKeyboardKey.pageDown), EditorInputKind.pageDown);
    });

    test('Ctrl with Backspace or Delete removes a word', () {
      expect(
        _kindFor(LogicalKeyboardKey.backspace, control: true),
        EditorInputKind.deleteWordBackward,
      );
      expect(
        _kindFor(LogicalKeyboardKey.delete, control: true),
        EditorInputKind.deleteWordForward,
      );
      expect(_kindFor(LogicalKeyboardKey.backspace), EditorInputKind.backspace);
    });

    test('Shift extends only caret motion', () {
      final shiftRight = EditorInputEvent.fromLogicalKey(
        LogicalKeyboardKey.arrowRight,
        shift: true,
      );
      expect(shiftRight!.extendsSelection, isTrue);

      final plainRight =
          EditorInputEvent.fromLogicalKey(LogicalKeyboardKey.arrowRight);
      expect(plainRight!.extendsSelection, isFalse);

      final shiftEnter = EditorInputEvent.fromLogicalKey(
        LogicalKeyboardKey.enter,
        shift: true,
      );
      expect(shiftEnter!.extendsSelection, isFalse);
    });
  });

  group('Caret motion', () {
    testWidgets('Home and End land on the line edges', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      await typeTextDirect(controller, 'alpha beta');

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.lineStart(),
      );
      expect(controller.caretOffset, 0);

      await sendEditorInputDirect(controller, const EditorInputEvent.lineEnd());
      expect(controller.caretOffset, 'alpha beta'.length);
    });

    testWidgets('Ctrl+Right steps to the start of the next word',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      await typeTextDirect(controller, 'alpha beta gamma');

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.documentStart(),
      );
      expect(controller.caretOffset, 0);

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.wordRight(),
      );
      expect(controller.caretOffset, 'alpha '.length);

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.wordRight(),
      );
      expect(controller.caretOffset, 'alpha beta '.length);
    });

    testWidgets('Ctrl+Left steps back a word', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      await typeTextDirect(controller, 'alpha beta');

      await sendEditorInputDirect(controller, const EditorInputEvent.wordLeft());
      expect(controller.caretOffset, 'alpha '.length);
    });

    testWidgets('Ctrl+End reaches the document tail', (tester) async {
      final engine = MockDocumentEngine(initialText: 'one two three');
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.ensureGlyphCaret();

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.documentEnd(),
      );
      expect(controller.caretOffset, 'one two three'.length);
    });

    testWidgets('Shift+End selects to the end of the line', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      await typeTextDirect(controller, 'alpha beta');

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.lineStart(),
      );
      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.lineEnd(shift: true),
      );

      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText, 'alpha beta');
    });
  });

  group('Editing keys', () {
    testWidgets('Shift+Enter breaks the line inside the paragraph',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      await typeTextDirect(controller, 'first');
      final runBefore = controller.caretRunId;

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.lineBreak(),
      );

      // A manual break stays in the same run — Enter would split into a new one.
      expect(controller.caretRunId, runBefore);
      expect(controller.documentText, contains('\n'));
    });

    testWidgets('Ctrl+Backspace removes the word before the caret',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      await typeTextDirect(controller, 'alpha beta');

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.deleteWordBackward(),
      );

      expect(controller.documentText, 'alpha ');
    });

    testWidgets('Ctrl+Delete removes the word after the caret', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      await typeTextDirect(controller, 'alpha beta');

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.documentStart(),
      );
      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.deleteWordForward(),
      );

      expect(controller.documentText, 'beta');
    });
  });

  group('Application chords', () {
    testWidgets('Ctrl+B, Ctrl+I and Ctrl+U toggle character formatting',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await tester.pumpWidget(
        MaterialApp(home: EditorScreen(controller: controller)),
      );
      await tester.pumpAndSettle();

      expect(controller.bold, isFalse);
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyB);
      expect(controller.bold, isTrue);

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyI);
      expect(controller.italic, isTrue);

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyU);
      expect(controller.underline, isTrue);
    });

    testWidgets('Ctrl+Z undoes typing without the macOS menu', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await tester.pumpWidget(
        MaterialApp(home: EditorScreen(controller: controller)),
      );
      await tester.pumpAndSettle();

      await typeTextDirect(controller, 'abc');
      expect(controller.documentText, 'abc');

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyZ);
      await controller.ensureLayoutReady();

      // One undo step per inserted character, and no stray "z" typed.
      expect(controller.documentText, 'ab');
    });

    testWidgets('Ctrl+Y redoes the undone edit', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await tester.pumpWidget(
        MaterialApp(home: EditorScreen(controller: controller)),
      );
      await tester.pumpAndSettle();

      await typeTextDirect(controller, 'abc');
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyZ);
      await controller.ensureLayoutReady();
      expect(controller.documentText, 'ab');

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyY);
      await controller.ensureLayoutReady();

      expect(controller.documentText, 'abc');
    });

    testWidgets('a Ctrl chord never types its letter', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await tester.pumpWidget(
        MaterialApp(home: EditorScreen(controller: controller)),
      );
      await tester.pumpAndSettle();
      controller.ensureGlyphCaret();

      for (final key in [
        LogicalKeyboardKey.keyB,
        LogicalKeyboardKey.keyL,
        LogicalKeyboardKey.keyY,
      ]) {
        await _sendCtrlChord(tester, key);
      }
      await controller.ensureLayoutReady();

      expect(controller.documentText, isEmpty);
    });

    testWidgets('alignment and line spacing chords reach the paragraph',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await tester.pumpWidget(
        MaterialApp(home: EditorScreen(controller: controller)),
      );
      await tester.pumpAndSettle();

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyE);
      expect(controller.alignment, TextAlign.center);

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyJ);
      expect(controller.alignment, TextAlign.justify);

      await _sendCtrlChord(tester, LogicalKeyboardKey.digit2);
      expect(controller.lineSpacing, LineSpacingMode.double_);
    });

    testWidgets('Ctrl+Shift+A and Ctrl+Shift+K stay distinct from Ctrl+A/K',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await tester.pumpWidget(
        MaterialApp(home: EditorScreen(controller: controller)),
      );
      await tester.pumpAndSettle();

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyA, shift: true);
      expect(controller.allCaps, isTrue);

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyK, shift: true);
      expect(controller.smallCaps, isTrue);
    });

    testWidgets('Ctrl+Shift+E turns track changes on', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await tester.pumpWidget(
        MaterialApp(home: EditorScreen(controller: controller)),
      );
      await tester.pumpAndSettle();

      expect(controller.trackChanges, isFalse);
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyE, shift: true);
      expect(controller.trackChanges, isTrue);
    });
  });

  group('macOS build — Cmd carries the chords', () {
    testWidgets('Cmd+B, Cmd+I and Cmd+U toggle character formatting',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await _onPlatform(TargetPlatform.macOS, () async {
        await tester.pumpWidget(
          MaterialApp(home: EditorScreen(controller: controller)),
        );
        await tester.pumpAndSettle();

        await _sendMetaChord(tester, LogicalKeyboardKey.keyB);
        expect(controller.bold, isTrue);

        await _sendMetaChord(tester, LogicalKeyboardKey.keyI);
        expect(controller.italic, isTrue);

        await _sendMetaChord(tester, LogicalKeyboardKey.keyU);
        expect(controller.underline, isTrue);
      });
    });

    testWidgets('Cmd+Shift+Z redoes what Cmd+Z undid', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await _onPlatform(TargetPlatform.macOS, () async {
        await tester.pumpWidget(
          MaterialApp(home: EditorScreen(controller: controller)),
        );
        await tester.pumpAndSettle();

        await typeTextDirect(controller, 'abc');
        await _sendMetaChord(tester, LogicalKeyboardKey.keyZ);
        await controller.ensureLayoutReady();
        expect(controller.documentText, 'ab');

        await _sendMetaChord(tester, LogicalKeyboardKey.keyZ, shift: true);
        await controller.ensureLayoutReady();
        expect(controller.documentText, 'abc');
      });
    });

    testWidgets('a Cmd chord never types its letter', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await _onPlatform(TargetPlatform.macOS, () async {
        await tester.pumpWidget(
          MaterialApp(home: EditorScreen(controller: controller)),
        );
        await tester.pumpAndSettle();
        controller.ensureGlyphCaret();

        await _sendMetaChord(tester, LogicalKeyboardKey.keyB);
        await _sendMetaChord(tester, LogicalKeyboardKey.keyL);
        await controller.ensureLayoutReady();

        expect(controller.documentText, isEmpty);
      });
    });
  });

  group('iOS build — the soft-keyboard overlay owns the keys', () {
    // iOS and Android omit the GlyphEditorSurface Focus, so every hardware
    // keystroke arrives at the hidden TextField's focus node instead.
    Future<List<WordChord>> pumpOverlay(
      WidgetTester tester,
      EditorController controller,
    ) async {
      final fired = <WordChord>[];
      await tester.pumpWidget(
        MaterialApp(
          home: Shortcuts(
            shortcuts: kWordChordShortcuts,
            child: Actions(
              actions: <Type, Action<Intent>>{
                WordChordIntent: CallbackAction<WordChordIntent>(
                  onInvoke: (intent) {
                    fired.add(intent.chord);
                    return null;
                  },
                ),
              },
              child: Material(
                child: Stack(
                  children: [WebGlyphTextInput(controller: controller)],
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      return fired;
    }

    testWidgets('Cmd chords bubble past the hidden text field', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await _onPlatform(TargetPlatform.iOS, () async {
        final fired = await pumpOverlay(tester, controller);

        await _sendMetaChord(tester, LogicalKeyboardKey.keyB);
        await _sendMetaChord(tester, LogicalKeyboardKey.keyS, shift: true);

        expect(fired, contains(WordChord.bold));
        expect(fired, contains(WordChord.saveAs));
      });
    });

    testWidgets('Cmd+Left and Cmd+Right reach the line edges', (tester) async {
      // An iPad Magic Keyboard has no Home/End, so Word uses Cmd+Arrow there.
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await _onPlatform(TargetPlatform.iOS, () async {
        await pumpOverlay(tester, controller);
        await typeTextDirect(controller, 'alpha beta');

        await _sendMetaChord(tester, LogicalKeyboardKey.arrowLeft);
        expect(controller.caretOffset, 0);

        await _sendMetaChord(tester, LogicalKeyboardKey.arrowRight);
        expect(controller.caretOffset, 'alpha beta'.length);
      });
    });

    testWidgets('Shift+Enter keeps the break inside the paragraph',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await _onPlatform(TargetPlatform.iOS, () async {
        await pumpOverlay(tester, controller);
        await typeTextDirect(controller, 'first');
        final runBefore = controller.caretRunId;

        await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
        await tester.sendKeyEvent(LogicalKeyboardKey.enter);
        await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
        await tester.pumpAndSettle();
        await controller.ensureLayoutReady();

        expect(controller.caretRunId, runBefore);
        expect(controller.documentText, contains('\n'));
      });
    });
  });

  group('Chord catalog', () {
    test('every spec registers both Ctrl and Cmd unless it is a function key',
        () {
      for (final spec in kWordChordSpecs) {
        expect(
          spec.activators.length,
          spec.bare ? 1 : 2,
          reason: '${spec.label} should bind Ctrl and Cmd',
        );
      }
    });

    test('no two specs claim the same activator', () {
      final seen = <String>{};
      for (final spec in kWordChordSpecs) {
        for (final activator in spec.activators) {
          final key = activator.toString();
          expect(
            seen.add(key),
            isTrue,
            reason: 'duplicate activator $key from ${spec.label}',
          );
        }
      }
      expect(kWordChordShortcuts.length, seen.length);
    });

    test('Word chords that must exist are all bound', () {
      final labels = kWiredShortcuts.map((s) => s.chord).toSet();
      for (final required in const [
        'Ctrl/Cmd+Z',
        'Ctrl/Cmd+Y',
        'Ctrl/Cmd+B',
        'Ctrl/Cmd+I',
        'Ctrl/Cmd+U',
        'Ctrl/Cmd+H',
        'Ctrl/Cmd+K',
        'Home, End',
        'Shift+Enter',
      ]) {
        expect(labels, contains(required));
      }
    });
  });
}
