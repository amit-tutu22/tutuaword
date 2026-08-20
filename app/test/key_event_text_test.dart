import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/key_event_text.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('printableCharacterFromKeyEvent', () {
    test('uses event.character when present', () {
      final event = KeyDownEvent(
        physicalKey: PhysicalKeyboardKey.keyA,
        logicalKey: LogicalKeyboardKey.keyA,
        character: 'x',
        timeStamp: Duration.zero,
      );
      expect(printableCharacterFromKeyEvent(event), 'x');
    });

    test('falls back to logical key for letters', () {
      final event = KeyDownEvent(
        physicalKey: PhysicalKeyboardKey.keyH,
        logicalKey: LogicalKeyboardKey.keyH,
        timeStamp: Duration.zero,
      );
      expect(printableCharacterFromKeyEvent(event), 'h');
    });

    test('falls back to shifted logical letter keys', () {
      final event = KeyDownEvent(
        physicalKey: PhysicalKeyboardKey.keyH,
        logicalKey: LogicalKeyboardKey.keyH,
        timeStamp: Duration.zero,
      );
      HardwareKeyboard.instance.handleKeyEvent(
        KeyDownEvent(
          physicalKey: PhysicalKeyboardKey.shiftLeft,
          logicalKey: LogicalKeyboardKey.shiftLeft,
          timeStamp: Duration.zero,
        ),
      );
      expect(printableCharacterFromKeyEvent(event), 'H');
      HardwareKeyboard.instance.handleKeyEvent(
        KeyUpEvent(
          physicalKey: PhysicalKeyboardKey.shiftLeft,
          logicalKey: LogicalKeyboardKey.shiftLeft,
          timeStamp: Duration.zero,
        ),
      );
    });

    test('Delete with DEL character is not printable', () {
      final event = KeyDownEvent(
        physicalKey: PhysicalKeyboardKey.delete,
        logicalKey: LogicalKeyboardKey.delete,
        character: '\u007f',
        timeStamp: Duration.zero,
      );
      expect(printableCharacterFromKeyEvent(event), isNull);
    });

    test('Tab character is not treated as printable text', () {
      final event = KeyDownEvent(
        physicalKey: PhysicalKeyboardKey.tab,
        logicalKey: LogicalKeyboardKey.tab,
        character: '\t',
        timeStamp: Duration.zero,
      );
      expect(printableCharacterFromKeyEvent(event), isNull);
    });

    test('Space is printable with or without a character payload', () {
      expect(
        printableCharacterFromKeyEvent(
          KeyDownEvent(
            physicalKey: PhysicalKeyboardKey.space,
            logicalKey: LogicalKeyboardKey.space,
            timeStamp: Duration.zero,
          ),
        ),
        ' ',
      );
      expect(
        printableCharacterFromKeyEvent(
          KeyDownEvent(
            physicalKey: PhysicalKeyboardKey.space,
            logicalKey: LogicalKeyboardKey.space,
            character: ' ',
            timeStamp: Duration.zero,
          ),
        ),
        ' ',
      );
    });

    test('Shift+Space still inserts a regular space', () {
      HardwareKeyboard.instance.handleKeyEvent(
        KeyDownEvent(
          physicalKey: PhysicalKeyboardKey.shiftLeft,
          logicalKey: LogicalKeyboardKey.shiftLeft,
          timeStamp: Duration.zero,
        ),
      );
      expect(
        printableCharacterFromKeyEvent(
          KeyDownEvent(
            physicalKey: PhysicalKeyboardKey.space,
            logicalKey: LogicalKeyboardKey.space,
            timeStamp: Duration.zero,
          ),
        ),
        ' ',
      );
      HardwareKeyboard.instance.handleKeyEvent(
        KeyUpEvent(
          physicalKey: PhysicalKeyboardKey.shiftLeft,
          logicalKey: LogicalKeyboardKey.shiftLeft,
          timeStamp: Duration.zero,
        ),
      );
    });

    test('Enter CR/LF payloads are not printable text', () {
      for (final payload in ['\r', '\n']) {
        expect(
          printableCharacterFromKeyEvent(
            KeyDownEvent(
              physicalKey: PhysicalKeyboardKey.enter,
              logicalKey: LogicalKeyboardKey.enter,
              character: payload,
              timeStamp: Duration.zero,
            ),
          ),
          isNull,
        );
      }
      expect(
        printableCharacterFromKeyEvent(
          KeyDownEvent(
            physicalKey: PhysicalKeyboardKey.numpadEnter,
            logicalKey: LogicalKeyboardKey.numpadEnter,
            character: '\n',
            timeStamp: Duration.zero,
          ),
        ),
        isNull,
      );
    });

    test('Backspace BS character is not printable', () {
      expect(
        printableCharacterFromKeyEvent(
          KeyDownEvent(
            physicalKey: PhysicalKeyboardKey.backspace,
            logicalKey: LogicalKeyboardKey.backspace,
            character: '\b',
            timeStamp: Duration.zero,
          ),
        ),
        isNull,
      );
    });
  });
}
