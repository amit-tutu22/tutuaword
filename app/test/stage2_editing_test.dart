import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';

/// Minimal v2 display list with one glyph (same shape as wysiwyg tests).
Uint8List _fakeGlyphDisplayList() {
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
    0, 0, 0, 0,
    0, 0, 0, 0,
    0, 0, 0, 0,
    0, 0, 0, 0,
  ];
  return Uint8List.fromList(parts);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('EditorController Stage 2 unit', () {
    late EditorController controller;

    setUp(() {
      controller = EditorController();
    });

    tearDown(() {
      controller.dispose();
    });

    test('engine-connected sessions prefer glyph editing', () {
      if (!controller.isEngineConnected) return;
      expect(controller.preferTextRendering, isFalse);
      expect(controller.usesGlyphRendering, isTrue);
    });

    test('mock sessions prefer text editing', () {
      if (controller.isEngineConnected) return;
      expect(controller.preferTextRendering, isTrue);
      expect(controller.usesGlyphRendering, isFalse);
    });

    test('setDisplayListForTest enables glyph mode flags', () {
      controller.setDisplayListForTest(_fakeGlyphDisplayList());
      expect(controller.preferTextRendering, isFalse);
      expect(controller.usesGlyphRendering, isTrue);
      expect(controller.displayListBytes, isNotEmpty);
    });

    test('formatting toggles update local toolbar state', () {
      controller.toggleBold();
      controller.toggleItalic();
      controller.toggleUnderline();
      controller.toggleStrikethrough();
      controller.setAlignment(TextAlign.center);
      controller.setFontSize(14);

      expect(controller.bold, isTrue);
      expect(controller.italic, isTrue);
      expect(controller.underline, isTrue);
      expect(controller.strikethrough, isTrue);
      expect(controller.alignment, TextAlign.center);
      expect(controller.fontSize, 14);
    });

    test('subscript and superscript are mutually exclusive', () {
      controller.toggleSubscript();
      expect(controller.subscript, isTrue);
      expect(controller.superscript, isFalse);

      controller.toggleSuperscript();
      expect(controller.superscript, isTrue);
      expect(controller.subscript, isFalse);
    });

    test('collapsed glyph selection reports no range', () {
      expect(controller.hasGlyphSelection, isFalse);
      expect(controller.selectionRects, isEmpty);
    });

    test('glyph insert is a no-op without caret or engine run', () {
      if (controller.isEngineConnected) return;
      // Mock mode: insertGlyphCharacter requires engine.
      controller.insertGlyphCharacter('A');
      expect(controller.documentText, isEmpty);
    });
  });

  group('GlyphEditorSurface Stage 2 unit', () {
    testWidgets('mounts and requests focus on page 0', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(_fakeGlyphDisplayList());

      final snapshot = DisplayListSnapshot.fromBytes(controller.displayListBytes);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: controller.pageWidth,
              height: controller.pageHeight,
              child: GlyphEditorSurface(
                controller: controller,
                pageIndex: 0,
                snapshot: snapshot,
                atlasImage: null,
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(GlyphEditorSurface), findsOneWidget);
    });

    testWidgets('tap on surface does not throw', (tester) async {
      final controller = EditorController();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(_fakeGlyphDisplayList());
      final snapshot = DisplayListSnapshot.fromBytes(controller.displayListBytes);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: controller.pageWidth,
              height: controller.pageHeight,
              child: GlyphEditorSurface(
                controller: controller,
                pageIndex: 0,
                snapshot: snapshot,
                atlasImage: null,
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byType(GlyphEditorSurface));
      await tester.pump();
    });
  });
}
