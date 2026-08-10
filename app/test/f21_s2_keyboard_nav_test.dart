import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/ui/keyboard_shortcuts.dart';
import 'package:tutuaword/ui/ribbon.dart';
import 'package:tutuaword/ui/ribbon_focusable.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

import 'editor_test_helpers.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('F21.S2 Keyboard navigation', () {
    testWidgets('I-F21-S2-ribbon-tab-order visits tabs then Home controls', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);

      await tester.binding.setSurfaceSize(const Size(1100, 400));
      addTearDown(() => tester.binding.setSurfaceSize(null));

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: WordRibbon(controller: controller),
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('word_ribbon')), findsOneWidget);
      expect(find.byKey(const Key('ribbon_tab_home')), findsOneWidget);
      expect(find.byKey(const Key('ribbon_tab_insert')), findsOneWidget);

      final homeNode = _focusNodeUnder(tester, find.byKey(const Key('ribbon_tab_home')));
      homeNode.requestFocus();
      await tester.pump();
      expect(homeNode.hasFocus, isTrue);

      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.pump();
      final insertNode =
          _focusNodeUnder(tester, find.byKey(const Key('ribbon_tab_insert')));
      expect(insertNode.hasFocus, isTrue);

      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.pump();
      final designNode =
          _focusNodeUnder(tester, find.byKey(const Key('ribbon_tab_design')));
      expect(designNode.hasFocus, isTrue);

      // Skip remaining tabs into the Home body.
      for (var i = 0; i < 8; i++) {
        await tester.sendKeyEvent(LogicalKeyboardKey.tab);
        await tester.pump();
      }
      final primary = FocusManager.instance.primaryFocus;
      expect(primary, isNotNull);
      expect(primary!.hasFocus, isTrue);
      expect(primary, isNot(same(homeNode)));
      expect(primary, isNot(same(insertNode)));
    });

    testWidgets('I-F21-S2-ribbon-activate Space invokes onPressed', (tester) async {
      var pressed = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FocusTraversalGroup(
              child: Center(
                child: RibbonLargeButton(
                  key: const Key('activate_target'),
                  icon: Icons.link,
                  label: 'Link',
                  onPressed: () => pressed++,
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      final node = _focusNodeUnder(tester, find.byKey(const Key('activate_target')));
      node.requestFocus();
      await tester.pump();
      expect(node.hasFocus, isTrue);
      expect(node.context, isNotNull);

      Actions.invoke(node.context!, const ActivateIntent());
      await tester.pump();
      expect(pressed, 1);
    });

    testWidgets('I-F21-S2-disabled-skip omits disabled controls', (tester) async {
      final enabledNode = FocusNode(debugLabel: 'enabled');
      final disabledNode = FocusNode(debugLabel: 'disabled');
      final nextNode = FocusNode(debugLabel: 'next');
      addTearDown(enabledNode.dispose);
      addTearDown(disabledNode.dispose);
      addTearDown(nextNode.dispose);

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FocusTraversalGroup(
              child: Row(
                children: [
                  RibbonFocusable(
                    key: const Key('enabled_btn'),
                    focusNode: enabledNode,
                    enabled: true,
                    onActivate: () {},
                    builder: (context, {required hovered, required focused}) =>
                        const SizedBox(width: 40, height: 40, child: Text('On')),
                  ),
                  RibbonFocusable(
                    key: const Key('disabled_btn'),
                    focusNode: disabledNode,
                    enabled: false,
                    onActivate: null,
                    builder: (context, {required hovered, required focused}) =>
                        const SizedBox(width: 40, height: 40, child: Text('Off')),
                  ),
                  RibbonFocusable(
                    key: const Key('enabled_btn_2'),
                    focusNode: nextNode,
                    enabled: true,
                    onActivate: () {},
                    builder: (context, {required hovered, required focused}) =>
                        const SizedBox(width: 40, height: 40, child: Text('On2')),
                  ),
                ],
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      enabledNode.requestFocus();
      await tester.pump();
      expect(enabledNode.hasFocus, isTrue);

      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.pump();

      expect(disabledNode.hasFocus, isFalse);
      expect(nextNode.hasFocus, isTrue);
    });

    test('D-F21-S2-shortcuts-doc matches kWiredShortcuts', () {
      final candidates = [
        File('docs/keyboard.md'),
        File('../docs/keyboard.md'),
      ];
      final docFile = candidates.firstWhere(
        (f) => f.existsSync(),
        orElse: () => candidates.last,
      );
      final doc = docFile.readAsStringSync();
      expect(doc, contains('F21.S2'));
      for (final shortcut in kWiredShortcuts) {
        expect(
          doc,
          contains(shortcut.chord),
          reason: 'docs/keyboard.md missing chord ${shortcut.chord}',
        );
      }
      expect(kWiredShortcuts.map((s) => s.id).toSet().length, kWiredShortcuts.length);
    });

    testWidgets('WordRibbon focusable detectors include enabled and disabled', (tester) async {
      final controller = createTestEditorController();
      addTearDown(controller.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: WordRibbon(controller: controller)),
        ),
      );
      expect(find.byKey(const Key('word_ribbon')), findsOneWidget);
      final detectors = find.byType(FocusableActionDetector);
      expect(detectors, findsWidgets);
      final enabledCount = tester
          .widgetList<FocusableActionDetector>(detectors)
          .where((d) => d.enabled)
          .length;
      final disabledCount = tester
          .widgetList<FocusableActionDetector>(detectors)
          .where((d) => !d.enabled)
          .length;
      expect(enabledCount, greaterThan(8));
      expect(disabledCount, greaterThan(0));
    });
  });
}

FocusNode _focusNodeUnder(WidgetTester tester, Finder ancestor) {
  final detectorFinder = find.descendant(
    of: ancestor,
    matching: find.byType(FocusableActionDetector),
  );
  expect(detectorFinder, findsWidgets);
  final detector = tester.widget<FocusableActionDetector>(detectorFinder.first);
  final node = detector.focusNode;
  expect(node, isNotNull, reason: 'RibbonFocusable should own a FocusNode');
  return node!;
}
