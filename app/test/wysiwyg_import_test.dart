import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';

/// Builds a minimal v2 display list with one glyph for widget tests.
Uint8List fakeGlyphDisplayList() {
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

  group('DisplayListSnapshot helpers', () {
    test('glyphCount and hasPaintableContent', () {
      final bytes = fakeGlyphDisplayList();
      final snapshot = DisplayListSnapshot.fromBytes(bytes);
      expect(snapshot.glyphCount, 1);
      expect(snapshot.hasPaintableGlyphs, isTrue);
      expect(snapshot.hasPaintableContent, isTrue);
    });
  });

  group('DocumentView glyph vs text mode', () {
    testWidgets('glyph mode paints CustomPaint not TextField', (tester) async {
      final controller = EditorController.forTest();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: DocumentView(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(CustomPaint), findsWidgets);
      expect(find.byType(TextField), findsNothing);
    });
  });

  group('DocumentView continuous scrolling', () {
    testWidgets('renders every page, not just the current one', (tester) async {
      tester.view.physicalSize = const Size(1400, 2400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.reset);

      final controller = EditorController.forTest();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 4);

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();

      expect(controller.pageCount, 4);
      expect(find.byKey(ValueKey('page-0-${controller.pageDisplayVersion(0)}')), findsOneWidget);
      expect(find.byKey(ValueKey('page-1-${controller.pageDisplayVersion(1)}')), findsOneWidget);
    });

    testWidgets('scrolling down advances the reported page', (tester) async {
      tester.view.physicalSize = const Size(800, 400);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.reset);

      final controller = EditorController.forTest();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 4);

      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: DocumentView(controller: controller))),
      );
      await tester.pumpAndSettle();
      expect(controller.currentPage, 0);
      expect(controller.pageCount, 4);

      final pageList = find.byKey(const ValueKey('document-page-list'));
      expect(pageList, findsOneWidget);
      final scrollable = find.descendant(
        of: pageList,
        matching: find.byType(Scrollable),
      );
      final position = tester.state<ScrollableState>(scrollable).position;
      expect(position.maxScrollExtent, greaterThan(0),
          reason: 'four injected pages must be taller than the viewport');
      position.jumpTo(position.maxScrollExtent);
      await tester.pump();

      expect(controller.currentPage, greaterThan(0),
          reason: 'scroll offset ${position.pixels} of ${position.maxScrollExtent}');
    });

    testWidgets('phone opens at 100% and fits page width without sideways pan',
        (tester) async {
      await _expectMobileFitsWidth(
        tester,
        platform: TargetPlatform.iOS,
        size: const Size(390, 844),
      );
    });

    testWidgets('tablet opens at 100% and fits page width without sideways pan',
        (tester) async {
      await _expectMobileFitsWidth(
        tester,
        platform: TargetPlatform.iOS,
        size: const Size(768, 1024),
      );
    });
  });

  group('EditorController render mode', () {
    test('forTest uses glyph rendering without TextField fallback', () {
      final controller = EditorController.forTest();
      addTearDown(controller.dispose);
      expect(controller.preferTextRendering, isFalse);
      expect(controller.usesGlyphRendering, isTrue);
      expect(controller.textController, isNull);
    });

    test('setDisplayListForTest injects display list bytes', () {
      final controller = EditorController.forTest();
      addTearDown(controller.dispose);
      controller.setDisplayListForTest(fakeGlyphDisplayList());
      expect(controller.displayListBytes, isNotEmpty);
      expect(controller.usesGlyphRendering, isTrue);
    });
  });
}

Finder _horizontalPan() {
  return find.byWidgetPredicate(
    (widget) =>
        widget is SingleChildScrollView &&
        widget.scrollDirection == Axis.horizontal,
  );
}

Future<void> _expectMobileFitsWidth(
  WidgetTester tester, {
  required TargetPlatform platform,
  required Size size,
}) async {
  debugDefaultTargetPlatformOverride = platform;
  tester.view.physicalSize = size;
  tester.view.devicePixelRatio = 1.0;
  try {
    final controller = EditorController.forTest();
    addTearDown(controller.dispose);
    controller.setDisplayListForTest(fakeGlyphDisplayList(), pageCount: 2);

    await tester.pumpWidget(
      MaterialApp(
        home: MediaQuery(
          data: MediaQueryData(size: size),
          child: Scaffold(body: DocumentView(controller: controller)),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(tester.takeException(), isNull);
    expect(controller.zoom, closeTo(1.0, 0.01));
    expect(
      find.byKey(ValueKey('page-0-${controller.pageDisplayVersion(0)}')),
      findsOneWidget,
    );
    expect(_horizontalPan(), findsNothing);

    final pageRect = tester.getRect(
      find.byKey(ValueKey('page-0-${controller.pageDisplayVersion(0)}')),
    );
    final viewRect = tester.getRect(find.byType(DocumentView));
    expect(pageRect.width, lessThanOrEqualTo(viewRect.width + 1.0));
    expect(pageRect.left, greaterThanOrEqualTo(viewRect.left - 1.0));
    expect(pageRect.right, lessThanOrEqualTo(viewRect.right + 1.0));
    // 100% on mobile fills the canvas width (shrink or grow), leaving only the
    // inter-page gap as side padding — not a letter-sized island on a tablet.
    expect(pageRect.width, greaterThan(viewRect.width * 0.85));
  } finally {
    debugDefaultTargetPlatformOverride = null;
    tester.view.reset();
  }
}
