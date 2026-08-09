import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/editor/recent_symbols.dart';
import 'package:tutuaword/editor/symbol_catalog.dart';

void main() {
  group('F15.S3 Recent symbols store', () {
    late RecentSymbolsStore store;

    setUp(() {
      store = RecentSymbolsStore(maxCount: 5);
    });

    test('U-F15-S3-record-moves-to-front', () {
      store.recordId('copyright');
      store.recordId('euro');
      store.recordId('copyright');

      expect(store.ids, ['copyright', 'euro']);
      expect(store.entries().map((e) => e.character), ['©', '€']);
    });

    test('U-F15-S3-record-character-resolves-catalog-id', () {
      store.recordCharacter('©');
      store.recordCharacter('€');

      expect(store.ids, ['euro', 'copyright']);
    });

    test('U-F15-S3-cap-trims-oldest', () {
      for (final id in ['copyright', 'euro', 'pound', 'yen', 'cent', 'won']) {
        store.recordId(id);
      }

      expect(store.ids.length, 5);
      expect(store.ids, isNot(contains('copyright')));
      expect(store.ids.first, 'won');
    });

    test('U-F15-S3-load-and-export-round-trip', () {
      store.recordId('alpha');
      store.recordId('grinning');
      store.loadIds(store.exportIds());

      expect(store.entries().map((e) => e.id), ['grinning', 'alpha']);
    });

    test('U-F15-S3-find-by-character', () {
      expect(SymbolCatalog.findByCharacter('©')?.id, 'copyright');
      expect(SymbolCatalog.findByCharacter('😀')?.id, 'grinning');
      expect(SymbolCatalog.findByCharacter('missing'), isNull);
    });

    test('U-F15-S3-unknown-character-fallback-id', () {
      store.recordCharacter('⌘');
      expect(store.ids.first, '_char:⌘');
      expect(store.entries().single.character, '⌘');
    });

    test('U-F15-S3-session-store-persists-symbol-ids', () async {
      final root = Directory('test/fixtures/f15_s3_session');
      if (root.existsSync()) root.deleteSync(recursive: true);
      root.createSync(recursive: true);
      addTearDown(() {
        if (root.existsSync()) root.deleteSync(recursive: true);
      });

      final sessionStore = DocumentSessionStore(root: root);
      await sessionStore.saveRecentSymbolIds(['pi', 'copyright', 'grinning']);

      expect(sessionStore.loadRecentSymbolIds(), ['pi', 'copyright', 'grinning']);

      final reloaded = RecentSymbolsStore();
      reloaded.loadIds(sessionStore.loadRecentSymbolIds());
      expect(
        reloaded.entries().map((e) => e.character),
        ['π', '©', '😀'],
      );
    });
  });
}
