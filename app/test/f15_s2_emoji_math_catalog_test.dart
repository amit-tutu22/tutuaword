import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/symbol_catalog.dart';

void main() {
  group('F15.S2 Emoji and math symbol catalog', () {
    test('U-F15-S2-emoji-category-has-grinning', () {
      final entry = SymbolCatalog.findById('grinning');
      expect(entry, isNotNull);
      expect(entry!.character, '😀');
      expect(entry.description, isNotEmpty);
    });

    test('U-F15-S2-emoji-symbols-non-empty', () {
      expect(SymbolCatalog.emojiSymbols(), isNotEmpty);
      for (final entry in SymbolCatalog.emojiSymbols()) {
        expect(entry.character.runes.length, greaterThanOrEqualTo(1));
        expect(entry.id, isNotEmpty);
      }
    });

    test('U-F15-S2-math-greek-has-alpha', () {
      final alpha = SymbolCatalog.findById('alpha');
      expect(alpha, isNotNull);
      expect(alpha!.character, 'α');
    });

    test('U-F15-S2-math-extended-includes-set-theory', () {
      final symbols = SymbolCatalog.mathExtendedSymbols();
      final chars = symbols.map((e) => e.character).toSet();
      expect(chars, containsAll(['∀', '∃', '∈', '∅']));
    });

    test('U-F15-S2-categories-include-emoji-and-greek', () {
      final ids = SymbolCatalog.categories.map((c) => c.id).toSet();
      expect(ids, containsAll(['emoji', 'math_greek']));
    });

    test('U-F15-S2-all-symbols-unique-ids', () {
      final ids = <String>{};
      for (final entry in SymbolCatalog.allSymbols()) {
        expect(ids.add(entry.id), isTrue, reason: 'duplicate id ${entry.id}');
      }
    });
  });
}
