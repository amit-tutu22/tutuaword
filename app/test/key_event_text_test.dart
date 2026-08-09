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
  });
}
