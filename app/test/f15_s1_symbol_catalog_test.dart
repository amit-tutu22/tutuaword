import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/editor/symbol_catalog.dart';

void main() {
  group('F15.S1 Symbol catalog', () {
    test('U-F15-S1-catalog-has-copyright', () {
      final entry = SymbolCatalog.findById('copyright');
      expect(entry, isNotNull);
      expect(entry!.character, '©');
    });

    test('U-F15-S1-categories-non-empty', () {
      expect(SymbolCatalog.categories, isNotEmpty);
      for (final category in SymbolCatalog.categories) {
        expect(category.symbols, isNotEmpty);
        expect(category.name, isNotEmpty);
      }
    });

    test('U-F15-S1-all-symbols-unique-ids', () {
      final ids = <String>{};
      for (final entry in SymbolCatalog.allSymbols()) {
        expect(ids.add(entry.id), isTrue, reason: 'duplicate id ${entry.id}');
        expect(entry.character, isNotEmpty);
      }
    });

    test('U-F15-S1-currency-includes-euro', () {
      final euro = SymbolCatalog.currency.symbols
          .where((e) => e.id == 'euro')
          .map((e) => e.character)
          .firstOrNull;
      expect(euro, '€');
    });
  });
}
