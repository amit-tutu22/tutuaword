import 'package:flutter/services.dart';

/// Resolves a single printable character from a key down event.
///
/// [KeyDownEvent.character] is often null on Flutter web; fall back to
/// [LogicalKeyboardKey.keyId] (USB usage) and shift maps.
String? printableCharacterFromKeyEvent(KeyEvent event) {
  if (event is! KeyDownEvent) return null;

  final fromCharacter = event.character;
  if (fromCharacter != null && fromCharacter.isNotEmpty) {
    final ch = fromCharacter.substring(0, 1);
    final code = ch.codeUnitAt(0);
    if (code == 0x09 || code >= 0x20) return ch;
  }

  return _characterFromLogicalKey(
    event.logicalKey,
    shift: HardwareKeyboard.instance.isShiftPressed,
  );
}

String? _characterFromLogicalKey(
  LogicalKeyboardKey key, {
  required bool shift,
}) {
  final id = key.keyId;

  // a-z (Flutter logical key ids match lowercase ASCII on web).
  if (id >= 0x61 && id <= 0x7a) {
    final ch = String.fromCharCode(id);
    return shift ? ch.toUpperCase() : ch;
  }

  // 0-9 row.
  if (id >= 0x30 && id <= 0x39) {
    if (!shift) return String.fromCharCode(id);
    return _shiftedDigit(id);
  }

  final numpadChar = _numpadCharacter(key);
  if (numpadChar != null) return numpadChar;

  // Unshifted punctuation often uses ASCII key ids directly.
  if (!shift && id >= 0x20 && id <= 0x7e) {
    return String.fromCharCode(id);
  }

  // Shifted punctuation (US QWERTY).
  if (shift) {
    const shifted = <int, String>{
      0x2d: '_',
      0x3d: '+',
      0x5b: '{',
      0x5d: '}',
      0x5c: '|',
      0x3b: ':',
      0x27: '"',
      0x2c: '<',
      0x2e: '>',
      0x2f: '?',
      0x60: '~',
    };
    final shiftedChar = shifted[id];
    if (shiftedChar != null) return shiftedChar;
  }

  final label = key.keyLabel;
  if (label.length == 1) return label;

  return null;
}

String? _numpadCharacter(LogicalKeyboardKey key) {
  if (key == LogicalKeyboardKey.numpad0) return '0';
  if (key == LogicalKeyboardKey.numpad1) return '1';
  if (key == LogicalKeyboardKey.numpad2) return '2';
  if (key == LogicalKeyboardKey.numpad3) return '3';
  if (key == LogicalKeyboardKey.numpad4) return '4';
  if (key == LogicalKeyboardKey.numpad5) return '5';
  if (key == LogicalKeyboardKey.numpad6) return '6';
  if (key == LogicalKeyboardKey.numpad7) return '7';
  if (key == LogicalKeyboardKey.numpad8) return '8';
  if (key == LogicalKeyboardKey.numpad9) return '9';
  if (key == LogicalKeyboardKey.numpadDecimal) return '.';
  return null;
}

String? _shiftedDigit(int id) {
  const map = <int, String>{
    0x31: '!',
    0x32: '@',
    0x33: '#',
    0x34: r'$',
    0x35: '%',
    0x36: '^',
    0x37: '&',
    0x38: '*',
    0x39: '(',
    0x30: ')',
  };
  return map[id];
}
