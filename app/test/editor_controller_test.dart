import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/editor_controller.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('EditorController (mock mode)', () {
    late EditorController controller;

    setUp(() {
      controller = EditorController();
    });

    tearDown(() {
      controller.dispose();
    });

    test('starts with one page and empty document', () {
      expect(controller.pageCount, 1);
      expect(controller.currentPage, 0);
      expect(controller.documentText, isEmpty);
    });

    test('insertCharacter appends text in mock mode', () {
      if (controller.isEngineConnected) return;
      controller.insertCharacter('H');
      controller.insertCharacter('i');
      expect(controller.documentText, 'Hi');
    });

    test('deleteBackward removes last character', () {
      if (controller.isEngineConnected) return;
      controller.insertCharacter('A');
      controller.insertCharacter('B');
      controller.deleteBackward();
      expect(controller.documentText, 'A');
    });

    test('deleteForward removes first character in mock mode', () {
      if (controller.isEngineConnected) return;
      controller.insertCharacter('A');
      controller.insertCharacter('B');
      controller.deleteForward();
      expect(controller.documentText, 'B');
    });

    test('textForPage returns full text on single page', () {
      if (controller.isEngineConnected) return;
      controller.insertCharacter('X');
      expect(controller.textForPage(0), 'X');
    });

    test('setCurrentPage clamps to valid range', () {
      controller.setCurrentPage(99);
      expect(controller.currentPage, 0);
    });

    test('replacePageText updates document on single page', () {
      controller.replacePageText(0, 'Updated');
      expect(controller.documentText, 'Updated');
    });

    test('togglePrintPreview flips flag', () {
      expect(controller.printPreview, isFalse);
      controller.togglePrintPreview();
      expect(controller.printPreview, isTrue);
      controller.togglePrintPreview();
      expect(controller.printPreview, isFalse);
    });

    test('toolbar stubs update status in mock mode', () {
      controller.insertTable();
      expect(controller.statusText, contains('Table'));

      controller.insertImage();
      expect(controller.statusText, contains('Image'));

      controller.applyHeading1();
      expect(controller.statusText, contains('Heading 1'));

      controller.applyBulletList();
      expect(controller.statusText, contains('Bullet'));
    });

    test('engine mode connects when FFI library is present', () {
      if (!controller.isEngineConnected) return;
      expect(controller.statusText, contains('Rust engine'));
    });

    test('toggleBold italic underline update local flags', () {
      controller.toggleBold();
      controller.toggleItalic();
      controller.toggleUnderline();
      expect(controller.bold, isTrue);
      expect(controller.italic, isTrue);
      expect(controller.underline, isTrue);
    });

    test('toggleTrackChanges flips flag', () {
      expect(controller.trackChanges, isFalse);
      controller.toggleTrackChanges();
      expect(controller.trackChanges, isTrue);
      controller.toggleTrackChanges();
      expect(controller.trackChanges, isFalse);
    });

    test('spellCheckDocument clears misspellings in mock mode', () async {
      await controller.spellCheckDocument();
      expect(controller.spellMisspellings, isEmpty);
      if (!controller.isEngineConnected) {
        expect(controller.statusText, contains('Spell check'));
      } else {
        expect(controller.statusText, anyOf(contains('Spell check'), contains('No spelling')));
      }
    });
  });

  group('EditorController ribbon state', () {
    late EditorController controller;

    setUp(() {
      controller = EditorController();
    });

    tearDown(() {
      controller.dispose();
    });

    test('defaults match Word ribbon expectations', () {
      expect(controller.fontFamily, 'Calibri');
      expect(controller.fontSize, 11);
      expect(controller.alignment, TextAlign.left);
      expect(controller.zoom, 1.0);
      expect(controller.showRuler, isFalse);
      expect(controller.showNavigationPane, isFalse);
      expect(controller.strikethrough, isFalse);
      expect(controller.subscript, isFalse);
      expect(controller.superscript, isFalse);
      expect(controller.allCaps, isFalse);
      expect(controller.smallCaps, isFalse);
      expect(controller.hidden, isFalse);
      expect(controller.ligatures, isTrue);
      expect(controller.documentTitle, 'Document1');
      expect(controller.wordCount, 0);
    });

    test('setFontFamily and setFontSize update state', () {
      controller.setFontFamily('Arial');
      controller.setFontSize(14);
      expect(controller.fontFamily, 'Arial');
      expect(controller.fontSize, 14);
    });

    test('font size clamps to valid range', () {
      controller.setFontSize(2);
      expect(controller.fontSize, 6);
      controller.setFontSize(200);
      expect(controller.fontSize, 96);
    });

    test('increaseFontSize and decreaseFontSize adjust by one', () {
      controller.increaseFontSize();
      expect(controller.fontSize, 12);
      controller.decreaseFontSize();
      expect(controller.fontSize, 11);
    });

    test('setAlignment updates paragraph alignment', () {
      controller.setAlignment(TextAlign.center);
      expect(controller.alignment, TextAlign.center);
      controller.setAlignment(TextAlign.justify);
      expect(controller.alignment, TextAlign.justify);
    });

    test('toggleStrikethrough flips flag', () {
      controller.toggleStrikethrough();
      expect(controller.strikethrough, isTrue);
      controller.toggleStrikethrough();
      expect(controller.strikethrough, isFalse);
    });

    test('subscript and superscript are mutually exclusive', () {
      controller.toggleSubscript();
      expect(controller.subscript, isTrue);
      expect(controller.superscript, isFalse);

      controller.toggleSuperscript();
      expect(controller.superscript, isTrue);
      expect(controller.subscript, isFalse);
    });

    test('all caps and small caps are mutually exclusive', () {
      controller.toggleAllCaps();
      expect(controller.allCaps, isTrue);
      expect(controller.smallCaps, isFalse);

      controller.toggleSmallCaps();
      expect(controller.smallCaps, isTrue);
      expect(controller.allCaps, isFalse);
    });

    test('toggleHidden and toggleLigatures flip flags', () {
      controller.toggleHidden();
      expect(controller.hidden, isTrue);
      controller.toggleLigatures();
      expect(controller.ligatures, isFalse);
    });

    test('setZoom clamps and zoomIn zoomOut work', () {
      controller.setZoom(0.1);
      expect(controller.zoom, 0.5);
      controller.setZoom(5.0);
      expect(controller.zoom, 3.0);

      controller.setZoom(1.0);
      controller.zoomIn();
      expect(controller.zoom, closeTo(1.1, 0.001));
      controller.zoomOut();
      expect(controller.zoom, closeTo(1.0, 0.001));
    });

    test('toggleRuler and toggleNavigationPane flip flags', () {
      controller.toggleRuler();
      expect(controller.showRuler, isTrue);
      controller.toggleNavigationPane();
      expect(controller.showNavigationPane, isTrue);
    });

    test('wordCount counts whitespace-separated tokens', () {
      controller.replacePageText(0, 'one two three');
      expect(controller.wordCount, 3);
      controller.replacePageText(0, '  spaced   words  ');
      expect(controller.wordCount, 2);
      controller.replacePageText(0, '');
      expect(controller.wordCount, 0);
    });

    test('clearInfoMessage is safe when no banner is shown', () {
      expect(controller.infoMessage, isNull);
      controller.clearInfoMessage();
      expect(controller.infoMessage, isNull);
    });
  });

  group('EditorController text pagination', () {
    late EditorController controller;

    setUp(() {
      controller = EditorController();
    });

    tearDown(() {
      controller.dispose();
    });

    test('resume-sized document paginates to a small page count', () {
      final paragraphs = List.generate(
        80,
        (i) =>
            'Section ${i + 1}: Professional experience and academic qualifications with detailed responsibilities.',
      );
      final text = paragraphs.join('\n\n');
      controller.replacePageText(0, text);

      expect(text.length, greaterThan(5000));
      expect(controller.pageCount, greaterThan(2));
      expect(controller.pageCount, lessThan(12));
    });

    test('textForPage fills pages with wrapped lines not single paragraphs', () {
      final text = List.generate(120, (i) => 'Line item number $i with some extra words.').join('\n');
      controller.replacePageText(0, text);

      expect(controller.pageCount, greaterThan(1));
      final firstPage = controller.textForPage(0);
      final firstPageLines = firstPage.split('\n').where((l) => l.isNotEmpty).length;
      expect(firstPageLines, greaterThan(5));
    });

    test('all pages together contain the full document text', () {
      const text = 'Alpha paragraph.\n\nBeta paragraph with more words.\n\nGamma end.';
      controller.replacePageText(0, text);

      final merged = List.generate(controller.pageCount, controller.textForPage).join('\n');
      expect(merged.replaceAll('\n', ' '), contains('Alpha paragraph'));
      expect(merged.replaceAll('\n', ' '), contains('Gamma end'));
    });
  });
}
