import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';

/// Creates an [EditorController] backed by [MockDocumentEngine] for tests.
EditorController createTestEditorController({MockDocumentEngine? engine}) =>
    EditorController.forTest(engine: engine);

/// Builds a minimal v2 display list with one glyph for widget tests.
///
/// When [rectCount] > 0, appends decoration rects after the glyph batch.
/// [rectColors] supplies ARGB values (defaults to opaque black per rect).
Uint8List fakeGlyphDisplayList({
  int rectCount = 0,
  List<int>? rectColors,
}) {
  final atlas = List<int>.filled(4 * 4 * 4, 0xFF);
  final parts = <int>[
    2, 0, 0, 0,
    1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0x44, 0x43,
    0, 0, 0x46, 0x43,
    4, 0, 0, 0,
    4, 0, 0, 0,
    atlas.length, 0, 0, 0,
    ...atlas,
    1, 0, 0, 0,
    72, 0, 0, 0, 100, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0,
    0, 0, 0, 0, // glyph color
  ];

  _writeU32(parts, rectCount);
  for (var i = 0; i < rectCount; i++) {
    _writeF32(parts, 72);
    _writeF32(parts, 100);
    _writeF32(parts, 50);
    _writeF32(parts, 4);
  }
  for (var i = 0; i < rectCount; i++) {
    final color =
        (rectColors != null && i < rectColors.length) ? rectColors[i] : 0xFF000000;
    _writeU32(parts, color);
  }
  // Empty path and image batches (v2 tail).
  _writeU32(parts, 0);
  _writeU32(parts, 0);

  return Uint8List.fromList(parts);
}

void _writeU32(List<int> parts, int value) {
  parts
    ..add(value & 0xFF)
    ..add((value >> 8) & 0xFF)
    ..add((value >> 16) & 0xFF)
    ..add((value >> 24) & 0xFF);
}

void _writeF32(List<int> parts, double value) {
  final bytes = ByteData(4)..setFloat32(0, value, Endian.little);
  parts.addAll(bytes.buffer.asUint8List());
}

/// Maps a single character to a [LogicalKeyboardKey] for widget keyboard tests.
LogicalKeyboardKey? logicalKeyForChar(String ch) {
  if (ch.length != 1) return null;
  const letterKeys = {
    'a': LogicalKeyboardKey.keyA,
    'b': LogicalKeyboardKey.keyB,
    'c': LogicalKeyboardKey.keyC,
    'd': LogicalKeyboardKey.keyD,
    'e': LogicalKeyboardKey.keyE,
    'f': LogicalKeyboardKey.keyF,
    'g': LogicalKeyboardKey.keyG,
    'h': LogicalKeyboardKey.keyH,
    'i': LogicalKeyboardKey.keyI,
    'j': LogicalKeyboardKey.keyJ,
    'k': LogicalKeyboardKey.keyK,
    'l': LogicalKeyboardKey.keyL,
    'm': LogicalKeyboardKey.keyM,
    'n': LogicalKeyboardKey.keyN,
    'o': LogicalKeyboardKey.keyO,
    'p': LogicalKeyboardKey.keyP,
    'q': LogicalKeyboardKey.keyQ,
    'r': LogicalKeyboardKey.keyR,
    's': LogicalKeyboardKey.keyS,
    't': LogicalKeyboardKey.keyT,
    'u': LogicalKeyboardKey.keyU,
    'v': LogicalKeyboardKey.keyV,
    'w': LogicalKeyboardKey.keyW,
    'x': LogicalKeyboardKey.keyX,
    'y': LogicalKeyboardKey.keyY,
    'z': LogicalKeyboardKey.keyZ,
  };
  final lower = ch.toLowerCase();
  final key = letterKeys[lower];
  if (key == null) return null;
  return ch == lower ? key : key; // shift handled by sendKeyEvent for uppercase
}

/// Pumps a wide surface suitable for ribbon widget tests.
Future<void> pumpWideRibbon(WidgetTester tester, Widget child) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  addTearDown(() => tester.binding.setSurfaceSize(null));
  await tester.pumpWidget(MaterialApp(home: Scaffold(body: child)));
}

/// Types [text] through [GlyphEditorSurface] keyboard handling.
Future<void> typeText(
  WidgetTester tester,
  EditorController controller,
  String text,
) async {
  controller.ensureGlyphCaret();
  for (final ch in text.split('')) {
    if (ch == ' ') {
      await tester.sendKeyEvent(LogicalKeyboardKey.space);
    } else if (ch == '\n') {
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    } else if (ch == '\t') {
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
    } else {
      final key = logicalKeyForChar(ch);
      if (key == null) continue;
      if (ch == ch.toUpperCase() && ch != ch.toLowerCase()) {
        await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
        await tester.sendKeyEvent(key);
        await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
      } else {
        await tester.sendKeyEvent(key);
      }
    }
    await tester.pump(const Duration(milliseconds: 20));
  }
  await controller.ensureLayoutReady();
  await tester.pumpAndSettle();
}

/// Inserts [text] directly via the controller (no widget keyboard).
Future<void> typeTextDirect(EditorController controller, String text) async {
  controller.ensureGlyphCaret();
  for (final ch in text.split('')) {
    if (ch == '\n') {
      await controller.insertGlyphParagraphBreak();
    } else {
      await controller.insertGlyphCharacter(ch);
    }
  }
  await controller.ensureLayoutReady();
}

/// Pumps [DocumentView] for [controller], optionally injecting display bytes.
Future<void> pumpTestDocumentView(
  WidgetTester tester,
  EditorController controller, {
  Uint8List? displayList,
  int pageCount = 1,
}) async {
  if (displayList != null) {
    controller.setDisplayListForTest(displayList, pageCount: pageCount);
  }
  await tester.pumpWidget(
    MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
  );
  await tester.pumpAndSettle();
}
