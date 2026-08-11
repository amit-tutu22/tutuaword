import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/glyph_editor_surface.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  Map<String, dynamic> engineCharFormat(MockDocumentEngine engine) {
    final json = jsonDecode(engine.fetchCaretFormat(engine.defaultRunId)!)
        as Map<String, dynamic>;
    return json['char_format'] as Map<String, dynamic>;
  }

  group('F03.S1 Core ribbon', () {
    /// I-F03-S1-font-size-caret-end: ribbon size at a collapsed caret after the
    /// last character must land on the engine run, not only the Flutter model.
    testWidgets('I-F03-S1-font-size-caret-end applies size to typed text', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await pumpTestDocumentView(tester, controller);
      await tester.tap(find.byType(GlyphEditorSurface).first);
      await tester.pump();

      await typeText(tester, controller, 'Hi');
      expect(controller.caretOffset, greaterThan(0), reason: 'caret should sit at run end');

      controller.setFontSize(24);
      await controller.ensureLayoutReady();
      await tester.pumpAndSettle();

      expect(controller.fontSize, 24);
      expect(controller.documentText.toLowerCase(), 'hi');
      expect(
        (engineCharFormat(engine)['font_size'] as num).toDouble(),
        24,
        reason: 'engine run must carry the ribbon font size',
      );
    });

    testWidgets('I-F03-S1-strike-super-sub toggles from Home tab', (tester) async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'x');

      await pumpWideRibbon(
        tester,
        SizedBox(height: 120, child: HomeTab(controller: controller)),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byTooltip('Strikethrough'));
      await controller.ensureLayoutReady();
      await tester.pump();
      expect(controller.strikethrough, isTrue);
      expect(engineCharFormat(engine)['strikethrough'], isTrue);

      await tester.tap(find.byTooltip('Subscript'));
      await controller.ensureLayoutReady();
      await tester.pump();
      expect(controller.subscript, isTrue);
      expect(controller.superscript, isFalse);
      expect(engineCharFormat(engine)['subscript'], isTrue);
      expect(engineCharFormat(engine)['superscript'], isFalse);

      await tester.tap(find.byTooltip('Superscript'));
      await controller.ensureLayoutReady();
      await tester.pump();
      expect(controller.superscript, isTrue);
      expect(controller.subscript, isFalse);
      expect(engineCharFormat(engine)['superscript'], isTrue);
      expect(engineCharFormat(engine)['subscript'], isFalse);
    });

    test('I-F03-S1-bold-toggle-roundtrip with engine caret', () async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'x');
      controller.toggleBold();
      await controller.ensureLayoutReady();
      expect(controller.bold, isTrue);
      expect(engineCharFormat(engine)['bold'], isTrue);

      controller.toggleBold();
      await controller.ensureLayoutReady();
      expect(controller.bold, isFalse);
      expect(engineCharFormat(engine)['bold'], isFalse);
    });

    test('I-F03-S1-italic-and-family apply on engine run', () async {
      final engine = MockDocumentEngine();
      final controller = createTestEditorController(engine: engine);
      addTearDown(controller.dispose);

      await typeTextDirect(controller, 'ab');
      controller.toggleItalic();
      controller.setFontFamily('Verdana');
      await controller.ensureLayoutReady();

      final fmt = engineCharFormat(engine);
      expect(controller.italic, isTrue);
      expect(controller.fontFamily, 'Verdana');
      expect(fmt['italic'], isTrue);
      expect(fmt['font_family'], 'Verdana');
    });
  });
}
