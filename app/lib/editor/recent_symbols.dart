import 'package:flutter/foundation.dart';
import 'package:tutuaword/editor/symbol_catalog.dart';

/// Tracks last-used symbols for the Insert Symbol dialog (F15.S3).
class RecentSymbolsStore {
  RecentSymbolsStore({this.maxCount = 12});

  static const recentCategoryId = 'recent';

  static RecentSymbolsStore instance = RecentSymbolsStore();

  final int maxCount;
  final List<String> _ids = [];

  List<String> get ids => List.unmodifiable(_ids);

  bool get isEmpty => _ids.isEmpty;

  /// Resolves stored ids to catalog entries (unknown ids are dropped).
  List<SymbolEntry> entries() {
    final resolved = <SymbolEntry>[];
    for (final id in _ids) {
      final entry = resolve(id);
      if (entry != null) resolved.add(entry);
    }
    return resolved;
  }

  SymbolEntry? resolve(String id) {
    if (id.startsWith('_char:')) {
      final character = id.substring('_char:'.length);
      if (character.isEmpty) return null;
      return SymbolEntry(id: id, character: character);
    }
    return SymbolCatalog.findById(id);
  }

  /// Records a symbol by catalog [id], moving it to the front (MRU).
  void recordId(String id) {
    if (id.isEmpty) return;
    _ids
      ..remove(id)
      ..insert(0, id);
    while (_ids.length > maxCount) {
      _ids.removeLast();
    }
  }

  /// Records a symbol by its inserted [character].
  void recordCharacter(String character) {
    if (character.isEmpty) return;
    final entry = SymbolCatalog.findByCharacter(character);
    if (entry != null) {
      recordId(entry.id);
    } else {
      recordId('_char:$character');
    }
  }

  /// Restores persisted ids (e.g. from session store).
  void loadIds(List<String> ids) {
    _ids
      ..clear()
      ..addAll(ids.take(maxCount));
  }

  List<String> exportIds() => List<String>.from(_ids);

  /// Clears all recents (for tests).
  void clear() => _ids.clear();

  @visibleForTesting
  static void resetInstanceForTest() {
    instance = RecentSymbolsStore();
  }
}
