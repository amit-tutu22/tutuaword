import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/plugin_registry.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F26.S3 WASM plugin host', () {
    test('U-F26-S3-capability-denied', () {
      final registry = PluginRegistry();
      registry.installSampleEditPlugin(grantEdit: false);
      final result = registry.invoke('com.tutuaword.sample.edit');
      expect(result, contains('capability denied'));
      expect(result, contains('Install sample'));
      expect(registry.lastError, contains('document.edit'));
    });

    test('U-F26-S3-capability-granted-invoke', () {
      final registry = PluginRegistry();
      registry.installSampleEditPlugin(grantEdit: true);
      final result = registry.invoke('com.tutuaword.sample.edit');
      expect(result, 'Hello from plugin');
      expect(registry.lastError, isNull);
    });

    test('U-F26-S3-lifecycle-disable', () {
      final registry = PluginRegistry();
      registry.installSampleEditPlugin(grantEdit: true);
      expect(registry.disable('com.tutuaword.sample.edit'), isTrue);
      expect(registry.invoke('com.tutuaword.sample.edit'), contains('disabled'));
      expect(registry.enable('com.tutuaword.sample.edit'), isTrue);
      expect(registry.invoke('com.tutuaword.sample.edit'), 'Hello from plugin');
    });

    testWidgets('I-F26-S3-add-ins-from-home', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpWideRibbon(
        tester,
        SizedBox(height: 140, child: HomeTab(controller: controller)),
      );
      await tester.scrollUntilVisible(
        find.byKey(const Key('home_add_ins')),
        120,
        scrollable: find.byType(Scrollable).first,
      );
      await tester.tap(find.byKey(const Key('home_add_ins')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('plugins_dialog')), findsOneWidget);
    });

    testWidgets('I-F26-S3-plugins-dialog-from-review', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await pumpRibbonTab(tester, ReviewTab(controller: controller));
      await tester.scrollUntilVisible(
        find.byKey(const Key('manage_plugins')),
        120,
        scrollable: find.byType(Scrollable),
      );
      await tester.tap(find.byKey(const Key('manage_plugins')));
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('plugins_dialog')), findsOneWidget);
      await tester.tap(find.byKey(const Key('plugins_install_sample')));
      await tester.pumpAndSettle();
      expect(controller.pluginRegistry.pluginCount, 1);

      await tester.tap(
        find.byKey(const Key('plugin_invoke_com.tutuaword.sample.edit')),
      );
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('plugins_status')), findsOneWidget);
      expect(find.text('Hello from plugin'), findsOneWidget);
    });

    testWidgets('I-F26-S3-capability-denied-in-dialog', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                key: const Key('open_plugins'),
                onPressed: () => controller.managePlugins(context),
                child: const Text('Plugins'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.byKey(const Key('open_plugins')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('plugins_install_sample_readonly')));
      await tester.pumpAndSettle();
      await tester.tap(
        find.byKey(const Key('plugin_invoke_com.tutuaword.sample.edit')),
      );
      await tester.pumpAndSettle();
      expect(
        find.textContaining('capability denied'),
        findsOneWidget,
      );
    });

    test('S-F26-S3-plugin-registry-churn', () {
      final registry = PluginRegistry();
      for (var i = 0; i < 200; i++) {
        registry.install(
          id: 'com.example.p$i',
          name: 'P$i',
          requested: const [
            PluginCapability.documentRead,
            PluginCapability.documentEdit,
          ],
          userGranted: i.isEven
              ? const [
                  PluginCapability.documentRead,
                  PluginCapability.documentEdit,
                ]
              : const [PluginCapability.documentRead],
        );
      }
      expect(registry.pluginCount, 200);
      var allowed = 0;
      var denied = 0;
      for (var i = 0; i < 200; i++) {
        final result = registry.invoke('com.example.p$i');
        if (result.startsWith('ERR:')) {
          denied++;
        } else {
          allowed++;
        }
      }
      expect(allowed, 100);
      expect(denied, 100);
    }, timeout: const Timeout(Duration(seconds: 30)));
  });
}
