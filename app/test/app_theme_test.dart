import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/ui/app_chrome_theme.dart';
import 'package:tutuaword/ui/app_theme_controller.dart';
import 'package:tutuaword/ui/ribbon.dart';
import 'package:tutuaword/ui/settings_dialog.dart';
import 'package:tutuaword/ui/title_bar.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('chrome accent persists without wiping autosave interval', () async {
    final root = Directory.systemTemp.createTempSync('tutuaword_chrome_');
    addTearDown(() => root.deleteSync(recursive: true));
    final store = DocumentSessionStore(root: root);

    await store.saveAutosaveInterval(const Duration(seconds: 15));
    await store.saveChromeAccent(const Color(0xFF0F766E));

    expect(store.loadAutosaveInterval(), const Duration(seconds: 15));
    expect(store.loadChromeAccent()?.toARGB32(), const Color(0xFF0F766E).toARGB32());

    await store.saveAutosaveInterval(const Duration(seconds: 45));
    expect(store.loadChromeAccent()?.toARGB32(), const Color(0xFF0F766E).toARGB32());
    expect(store.loadAutosaveInterval(), const Duration(seconds: 45));
  });

  testWidgets('settings dialog lists accents and applies teal', (tester) async {
    final root = Directory.systemTemp.createTempSync('tutuaword_settings_ui_');
    addTearDown(() => root.deleteSync(recursive: true));
    final store = DocumentSessionStore(root: root);
    final theme = AppThemeController(store: store);

    await tester.pumpWidget(
      ListenableBuilder(
        listenable: theme,
        builder: (context, _) => MaterialApp(
          theme: buildTutuawordTheme(theme.accent),
          home: Scaffold(
            body: Builder(
              builder: (context) => TextButton(
                onPressed: () => AppSettingsDialog.show(context, theme: theme),
                child: const Text('open'),
              ),
            ),
          ),
        ),
      ),
    );

    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();

    expect(find.byKey(const Key('app_settings_dialog')), findsOneWidget);
    expect(find.byKey(const Key('chrome_accent_blue')), findsOneWidget);
    expect(find.byKey(const Key('chrome_accent_teal')), findsOneWidget);

    await tester.tap(find.byKey(const Key('chrome_accent_teal')));
    await tester.pumpAndSettle();

    expect(theme.accent.toARGB32(), const Color(0xFF0F766E).toARGB32());
    expect(store.loadChromeAccent()?.toARGB32(), const Color(0xFF0F766E).toARGB32());
  });

  testWidgets('themed chrome tints title bar and ribbon', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);
    const accent = Color(0xFF6D28D9);
    final chrome = TutuawordChrome.fromAccent(accent);

    await tester.binding.setSurfaceSize(const Size(1400, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      MaterialApp(
        theme: buildTutuawordTheme(accent),
        home: MediaQuery(
          data: const MediaQueryData(size: Size(1400, 900)),
          child: Scaffold(
            body: Column(
              children: [
                WordTitleBar(controller: controller),
                WordRibbon(controller: controller),
              ],
            ),
          ),
        ),
      ),
    );

    final title = tester.widget<Container>(find.byKey(const Key('title_bar_chrome')));
    expect(title.color?.toARGB32(), chrome.titleBar.toARGB32());
    final ribbon = tester.widget<Container>(find.byKey(const Key('ribbon_body')));
    expect(ribbon.color?.toARGB32(), chrome.ribbonSurface.toARGB32());
  });

  testWidgets('View tab opens settings', (tester) async {
    final controller = createTestEditorController();
    addTearDown(controller.dispose);
    final root = Directory.systemTemp.createTempSync('tutuaword_view_settings_');
    addTearDown(() => root.deleteSync(recursive: true));
    final theme = AppThemeController(store: DocumentSessionStore(root: root));

    await tester.binding.setSurfaceSize(const Size(1400, 900));
    addTearDown(() => tester.binding.setSurfaceSize(null));
    await tester.pumpWidget(
      MaterialApp(
        theme: buildTutuawordTheme(theme.accent),
        home: MediaQuery(
          data: const MediaQueryData(size: Size(1400, 900)),
          child: Scaffold(body: WordRibbon(controller: controller)),
        ),
      ),
    );

    await tester.tap(find.byKey(const Key('ribbon_tab_view')));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('view_settings')), findsOneWidget);

    await tester.ensureVisible(find.byKey(const Key('view_settings')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('view_settings')));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('app_settings_dialog')), findsOneWidget);
  });
}
