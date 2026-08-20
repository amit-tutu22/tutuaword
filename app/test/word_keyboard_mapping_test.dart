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

/// Presses [key] with an arbitrary modifier set, for chords that carry no
/// Ctrl/Cmd at all (Word's function keys and its Alt+Shift outline chords).
Future<void> _sendChord(
  WidgetTester tester,
  LogicalKeyboardKey key, {
  bool control = false,
  bool meta = false,
  bool shift = false,
  bool alt = false,
}) async {
  if (control) await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
  if (meta) await tester.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
  if (shift) await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
  if (alt) await tester.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
  await tester.sendKeyEvent(key);
  if (alt) await tester.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
  if (shift) await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
  if (meta) await tester.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
  if (control) await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
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
      expect(_kindFor(LogicalKeyboardKey.numpadEnter), EditorInputKind.newline);
    });

    test('Space is a character, not an editing intent', () {
      expect(_kindFor(LogicalKeyboardKey.space), isNull);
      expect(
        EditorInputEvent.fromLogicalKey(
          LogicalKeyboardKey.space,
          character: ' ',
        )?.kind,
        EditorInputKind.character,
      );
    });

    test('Enter CR/LF character payloads map to newline', () {
      expect(
        EditorInputEvent.fromLogicalKey(
          LogicalKeyboardKey.enter,
          character: '\r',
        )?.kind,
        EditorInputKind.newline,
      );
      expect(
        EditorInputEvent.fromLogicalKey(
          LogicalKeyboardKey.keyA,
          character: '\n',
        )?.kind,
        EditorInputKind.newline,
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
      expect(_kindFor(LogicalKeyboardKey.delete), EditorInputKind.delete);
    });

    test('Tab and Shift+Tab are editing intents', () {
      expect(_kindFor(LogicalKeyboardKey.tab), EditorInputKind.tab);
      expect(
        EditorInputEvent.fromLogicalKey(LogicalKeyboardKey.tab, shift: true)!.shift,
        isTrue,
      );
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

  group('Chords wired on top of the first pass', () {
    /// Mounts only the `Shortcuts` map, so a chord that resolves to the wrong
    /// intent — or to none — is visible without any controller state.
    Future<List<WordChord>> pumpChordProbe(WidgetTester tester) async {
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
              child: const Focus(autofocus: true, child: SizedBox()),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      return fired;
    }

    testWidgets('each new chord resolves to its own command', (tester) async {
      final fired = await pumpChordProbe(tester);

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyC, shift: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyV, shift: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyV, alt: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyX, shift: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyH, shift: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyL, shift: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.bracketRight);
      await _sendCtrlChord(tester, LogicalKeyboardKey.bracketLeft);
      await _sendCtrlChord(tester, LogicalKeyboardKey.space, shift: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.minus);
      await _sendCtrlChord(tester, LogicalKeyboardKey.minus, shift: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyC, alt: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyR, alt: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyT, alt: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.period, alt: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyF, alt: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyD, alt: true);
      await _sendCtrlChord(tester, LogicalKeyboardKey.f2);
      await _sendChord(tester, LogicalKeyboardKey.f3, shift: true);
      await _sendChord(tester, LogicalKeyboardKey.f7);
      await _sendChord(tester, LogicalKeyboardKey.f12);
      await _sendChord(tester, LogicalKeyboardKey.keyD, shift: true, alt: true);
      await _sendChord(tester, LogicalKeyboardKey.keyP, shift: true, alt: true);
      await _sendChord(
        tester,
        LogicalKeyboardKey.arrowLeft,
        shift: true,
        alt: true,
      );
      await _sendChord(
        tester,
        LogicalKeyboardKey.arrowRight,
        shift: true,
        alt: true,
      );

      expect(fired, <WordChord>[
        WordChord.copyFormatting,
        WordChord.pasteFormatting,
        WordChord.pasteSpecial,
        WordChord.strikethrough,
        WordChord.hiddenText,
        WordChord.bulletList,
        WordChord.growFont,
        WordChord.shrinkFont,
        WordChord.nonbreakingSpace,
        WordChord.optionalHyphen,
        WordChord.nonbreakingHyphen,
        WordChord.copyright,
        WordChord.registered,
        WordChord.trademark,
        WordChord.ellipsis,
        WordChord.footnote,
        WordChord.endnote,
        WordChord.printPreview,
        WordChord.changeCase,
        WordChord.spelling,
        WordChord.saveAs,
        WordChord.dateField,
        WordChord.pageNumberField,
        WordChord.promoteOutline,
        WordChord.demoteOutline,
      ]);
    });

    testWidgets('Ctrl+Shift+X and Ctrl+Shift+H reach the run format',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await tester.pumpWidget(
        MaterialApp(home: EditorScreen(controller: controller)),
      );
      await tester.pumpAndSettle();

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyX, shift: true);
      expect(controller.strikethrough, isTrue);

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyH, shift: true);
      expect(controller.hidden, isTrue);
    });

    testWidgets('Ctrl+Alt+C types a copyright sign, not a "c"', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await tester.pumpWidget(
        MaterialApp(home: EditorScreen(controller: controller)),
      );
      await tester.pumpAndSettle();
      controller.ensureGlyphCaret();

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyC, alt: true);
      await controller.ensureLayoutReady();

      expect(controller.documentText, '\u00A9');
    });

    testWidgets('Shift+F3 walks lower → Title → UPPER → lower', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      await typeTextDirect(controller, 'alpha beta');
      await controller.selectAll();

      await controller.cycleChangeCase();
      expect(controller.documentText, 'Alpha Beta');

      await controller.cycleChangeCase();
      expect(controller.documentText, 'ALPHA BETA');

      await controller.cycleChangeCase();
      expect(controller.documentText, 'alpha beta');
    });

    test('Alt+Shift+Arrow is an outline chord off Apple platforms', () {
      // The editor surface must let it through so the list re-levels; on macOS
      // and iPadOS the OS convention wins and it stays word-wise selection.
      debugDefaultTargetPlatformOverride = TargetPlatform.windows;
      try {
        expect(
          _kindFor(LogicalKeyboardKey.arrowLeft, shift: true, alt: true),
          isNull,
        );
        expect(
          _kindFor(LogicalKeyboardKey.arrowLeft, alt: true),
          EditorInputKind.wordLeft,
        );
      } finally {
        debugDefaultTargetPlatformOverride = null;
      }

      debugDefaultTargetPlatformOverride = TargetPlatform.macOS;
      try {
        expect(
          _kindFor(LogicalKeyboardKey.arrowRight, shift: true, alt: true),
          EditorInputKind.wordRight,
        );
      } finally {
        debugDefaultTargetPlatformOverride = null;
      }
    });
  });

  group('Paragraph motion and paragraph-level chords', () {
    /// Three paragraphs the mock engine reports as newline-separated text.
    EditorController threeParagraphController() {
      final controller = createTestEditorController(
        engine: MockDocumentEngine(initialText: 'alpha\nbeta\ngamma'),
      );
      controller.ensureGlyphCaret();
      return controller;
    }

    testWidgets('Ctrl+Up goes to this paragraph start, then the previous one',
        (tester) async {
      final controller = threeParagraphController();
      addTearDown(controller.dispose);
      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.documentEnd(),
      );
      expect(controller.caretOffset, 'alpha\nbeta\ngamma'.length);

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.paragraphUp(),
      );
      expect(controller.caretOffset, 'alpha\nbeta\n'.length);

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.paragraphUp(),
      );
      expect(controller.caretOffset, 'alpha\n'.length);
    });

    testWidgets('Ctrl+Down lands on the next paragraph start', (tester) async {
      final controller = threeParagraphController();
      addTearDown(controller.dispose);
      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.documentStart(),
      );

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.paragraphDown(),
      );
      expect(controller.caretOffset, 'alpha\n'.length);

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.paragraphDown(),
      );
      expect(controller.caretOffset, 'alpha\nbeta\n'.length);
    });

    testWidgets('Ctrl+Shift+Down selects to the end of the paragraph',
        (tester) async {
      final controller = threeParagraphController();
      addTearDown(controller.dispose);
      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.documentStart(),
      );

      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.paragraphDown(shift: true),
      );

      expect(controller.hasGlyphSelection, isTrue);
      expect(controller.selectedText, 'alpha\n');
    });

    test('Ctrl+Arrow and Option+Arrow both mean paragraph motion', () {
      expect(
        _kindFor(LogicalKeyboardKey.arrowUp, control: true),
        EditorInputKind.paragraphUp,
      );
      expect(
        _kindFor(LogicalKeyboardKey.arrowDown, control: true),
        EditorInputKind.paragraphDown,
      );
      expect(
        _kindFor(LogicalKeyboardKey.arrowUp, shift: true),
        EditorInputKind.arrowUp,
      );

      debugDefaultTargetPlatformOverride = TargetPlatform.macOS;
      try {
        expect(
          _kindFor(LogicalKeyboardKey.arrowDown, alt: true),
          EditorInputKind.paragraphDown,
        );
      } finally {
        debugDefaultTargetPlatformOverride = null;
      }
    });

    test('Alt+Shift+Up/Down is a command chord on every platform', () {
      for (final platform in const [
        TargetPlatform.windows,
        TargetPlatform.macOS,
        TargetPlatform.iOS,
      ]) {
        debugDefaultTargetPlatformOverride = platform;
        try {
          expect(
            _kindFor(LogicalKeyboardKey.arrowUp, shift: true, alt: true),
            isNull,
            reason: 'Alt+Shift+Up must reach the move-paragraph chord',
          );
          expect(
            _kindFor(LogicalKeyboardKey.arrowDown, shift: true, alt: true),
            isNull,
          );
        } finally {
          debugDefaultTargetPlatformOverride = null;
        }
      }
    });

    testWidgets('Alt+Shift+Up moves the paragraph above its neighbour',
        (tester) async {
      final controller = threeParagraphController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await tester.pumpWidget(
        MaterialApp(home: EditorScreen(controller: controller)),
      );
      await tester.pumpAndSettle();
      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.documentEnd(),
      );

      await _sendChord(
        tester,
        LogicalKeyboardKey.arrowUp,
        shift: true,
        alt: true,
      );
      await controller.ensureLayoutReady();

      expect(controller.documentText, 'alpha\ngamma\nbeta');
    });

    testWidgets('Alt+Shift+Down moves it back, and undo restores the order',
        (tester) async {
      final controller = threeParagraphController();
      addTearDown(controller.dispose);
      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.documentStart(),
      );

      await controller.moveParagraph(direction: 1);
      await controller.ensureLayoutReady();
      expect(controller.documentText, 'beta\nalpha\ngamma');

      await controller.undo();
      await controller.ensureLayoutReady();
      expect(controller.documentText, 'alpha\nbeta\ngamma');
    });

    testWidgets('moving past the first paragraph reports the edge',
        (tester) async {
      final controller = threeParagraphController();
      addTearDown(controller.dispose);
      await sendEditorInputDirect(
        controller,
        const EditorInputEvent.documentStart(),
      );

      await controller.moveParagraph(direction: -1);
      await controller.ensureLayoutReady();

      expect(controller.documentText, 'alpha\nbeta\ngamma');
      expect(controller.statusText, contains('already at the edge'));
    });

    testWidgets('Ctrl+T hangs the first line, Ctrl+Shift+T pulls it back',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await tester.pumpWidget(
        MaterialApp(home: EditorScreen(controller: controller)),
      );
      await tester.pumpAndSettle();

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyT);
      expect(controller.indentLeft, 36);
      expect(controller.indentFirstLine, -36);

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyT, shift: true);
      expect(controller.indentLeft, 0);
      expect(controller.indentFirstLine, 0);
    });

    testWidgets('Ctrl+0 toggles the 12 pt space above the paragraph',
        (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await tester.pumpWidget(
        MaterialApp(home: EditorScreen(controller: controller)),
      );
      await tester.pumpAndSettle();

      await _sendCtrlChord(tester, LogicalKeyboardKey.digit0);
      expect(controller.spaceBefore, 12);

      await _sendCtrlChord(tester, LogicalKeyboardKey.digit0);
      expect(controller.spaceBefore, 0);
    });

    testWidgets('Ctrl+Q drops the direct paragraph formatting', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      await tester.pumpWidget(
        MaterialApp(home: EditorScreen(controller: controller)),
      );
      await tester.pumpAndSettle();

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyE);
      await _sendCtrlChord(tester, LogicalKeyboardKey.keyM);
      await _sendCtrlChord(tester, LogicalKeyboardKey.digit2);
      expect(controller.alignment, TextAlign.center);
      expect(controller.indentLeft, 36);
      expect(controller.lineSpacing, LineSpacingMode.double_);

      await _sendCtrlChord(tester, LogicalKeyboardKey.keyQ);

      expect(controller.alignment, TextAlign.left);
      expect(controller.indentLeft, 0);
      expect(controller.spaceBefore, 0);
      expect(controller.lineSpacing, LineSpacingMode.single);
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
        'Ctrl/Cmd+Shift+C',
        'Ctrl/Cmd+Shift+V',
        'Shift+F3',
        'Alt+Shift+Left',
        'Alt+Shift+Right',
        'Ctrl/Cmd+Shift+Space',
        'F7',
        'F12',
        'Alt+Shift+Up',
        'Alt+Shift+Down',
        'Ctrl/Cmd+Q',
        'Ctrl/Cmd+T',
        'Ctrl/Cmd+Shift+T',
        'Ctrl/Cmd+0',
        'Ctrl+Up, Ctrl+Down',
        'F1',
      ]) {
        expect(labels, contains(required));
      }
    });
  });
}
