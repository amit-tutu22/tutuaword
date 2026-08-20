import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/ui/keyboard_help_dialog.dart';
import 'package:tutuaword/ui/keyboard_shortcuts.dart';

void main() {
  group('Keyboard help dialog', () {
    test('catalog covers every wired chord and editor action', () {
      final catalogChords = buildKeyboardHelpSections()
          .expand((section) => section.rows)
          .map((row) => '${row.chord}|${row.action}')
          .toSet();

      for (final spec in kWordChordSpecs) {
        expect(
          catalogChords,
          contains('${spec.label}|${spec.action}'),
          reason: '${spec.label} must appear in the Help dialog',
        );
      }
      for (final entry in kEditorKeyActions.entries) {
        expect(
          catalogChords,
          contains('${entry.key}|${entry.value}'),
          reason: 'editor ${entry.key} must appear in the Help dialog',
        );
      }
    });

    testWidgets('search filters chords and F1 opens the dialog via Shortcuts',
        (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: Shortcuts(
                shortcuts: kWordChordShortcuts,
                child: Actions(
                  actions: <Type, Action<Intent>>{
                    WordChordIntent: CallbackAction<WordChordIntent>(
                      onInvoke: (intent) {
                        if (intent.chord == WordChord.keyboardHelp) {
                          KeyboardHelpDialog.show(context);
                        }
                        return null;
                      },
                    ),
                  },
                  child: const Focus(
                    autofocus: true,
                    child: SizedBox.expand(),
                  ),
                ),
              ),
            ),
          ),
        ),
      );

      await tester.sendKeyEvent(LogicalKeyboardKey.f1);
      await tester.pumpAndSettle();

      expect(find.byKey(const Key('keyboard_help_dialog')), findsOneWidget);
      expect(find.text('Keyboard shortcuts'), findsOneWidget);
      expect(find.text('File'), findsOneWidget);
      expect(find.textContaining('Ctrl/Cmd+N'), findsWidgets);
      expect(find.textContaining('New document'), findsWidgets);

      await tester.enterText(
        find.byKey(const Key('keyboard_help_search')),
        'hanging',
      );
      await tester.pumpAndSettle();

      expect(find.textContaining('hanging indent'), findsWidgets);
      expect(find.text('File'), findsNothing);

      await tester.tap(find.byKey(const Key('keyboard_help_close')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('keyboard_help_dialog')), findsNothing);
    });
  });
}
