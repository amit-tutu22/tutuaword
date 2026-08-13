import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon.dart';
import 'package:tutuaword/ui/status_bar.dart';
import 'package:tutuaword/ui/title_bar.dart';
import 'package:tutuaword/ui/word_theme.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  Future<void> pumpDesktop(
    WidgetTester tester,
    TargetPlatform platform,
    Widget child,
  ) async {
    debugDefaultTargetPlatformOverride = platform;
    await tester.binding.setSurfaceSize(const Size(1400, 900));
    await tester.pumpWidget(
      MaterialApp(
        home: MediaQuery(
          data: const MediaQueryData(size: Size(1400, 900)),
          child: Scaffold(body: child),
        ),
      ),
    );
  }

  for (final platform in [
    TargetPlatform.macOS,
    TargetPlatform.windows,
    TargetPlatform.linux,
  ]) {
    group('Desktop regression (${platform.name})', () {
      late EditorController controller;

      setUp(() {
        controller = EditorController.forTest();
        controller.ensureGlyphCaret();
      });

      tearDown(() {
        controller.dispose();
      });

      testWidgets('uses desktop chrome helpers', (tester) async {
        debugDefaultTargetPlatformOverride = platform;
        try {
          await tester.pumpWidget(
            MaterialApp(
              home: MediaQuery(
                data: const MediaQueryData(size: Size(1400, 900)),
                child: Builder(
                  builder: (context) {
                    expect(WordTheme.phoneChrome(context), isFalse);
                    expect(WordTheme.tabletChrome(context), isFalse);
                    expect(WordTheme.mobileChrome(context), isFalse);
                    expect(WordTheme.ribbonHeightFor(context), 92.0);
                    if (platform == TargetPlatform.macOS) {
                      expect(WordTheme.leadingChromeInset(context), 78.0);
                    } else {
                      expect(WordTheme.leadingChromeInset(context), 8.0);
                    }
                    return const SizedBox();
                  },
                ),
              ),
            ),
          );
        } finally {
          debugDefaultTargetPlatformOverride = null;
        }
      });

      testWidgets('title bar shows full QAT without overflow menu', (tester) async {
        try {
          await pumpDesktop(tester, platform, WordTitleBar(controller: controller));
          expect(find.byIcon(Icons.undo), findsOneWidget);
          expect(find.byIcon(Icons.print_outlined), findsOneWidget);
          expect(find.byKey(const Key('title_bar_overflow')), findsNothing);
          expect(tester.takeException(), isNull);
        } finally {
          debugDefaultTargetPlatformOverride = null;
          await tester.binding.setSurfaceSize(null);
        }
      });

      testWidgets('ribbon keeps Share label and full height', (tester) async {
        try {
          await pumpDesktop(tester, platform, WordRibbon(controller: controller));
          expect(find.text('Share'), findsOneWidget);
          expect(find.text('Paste'), findsOneWidget);
          expect(tester.takeException(), isNull);
        } finally {
          debugDefaultTargetPlatformOverride = null;
          await tester.binding.setSurfaceSize(null);
        }
      });

      testWidgets('status bar keeps slider and view mode icons', (tester) async {
        try {
          await pumpDesktop(tester, platform, WordStatusBar(controller: controller));
          expect(find.byType(Slider), findsOneWidget);
          expect(find.byIcon(Icons.article_outlined), findsOneWidget);
          expect(find.byKey(const Key('status_view_menu')), findsNothing);
          expect(tester.takeException(), isNull);
        } finally {
          debugDefaultTargetPlatformOverride = null;
          await tester.binding.setSurfaceSize(null);
        }
      });

      testWidgets('document canvas centers page without phone horizontal pan', (tester) async {
        try {
          await pumpDesktop(tester, platform, DocumentView(controller: controller));
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull);
          expect(find.byType(FittedBox), findsWidgets);
          // Phone-only pan wrapper should not be required at 1400px desktop width.
          expect(controller.zoom, 1.0);
          final page = find.byKey(
            ValueKey('page-0-${controller.pageDisplayVersion(0)}'),
          );
          expect(page, findsOneWidget);
          final pageCenter = tester.getCenter(page);
          final view = tester.getRect(find.byType(DocumentView));
          expect(pageCenter.dx, closeTo(view.center.dx, 40.0));
        } finally {
          debugDefaultTargetPlatformOverride = null;
          await tester.binding.setSurfaceSize(null);
        }
      });
    });
  }
}
