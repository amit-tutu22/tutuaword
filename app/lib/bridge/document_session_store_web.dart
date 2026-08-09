import 'dart:convert';
import 'dart:typed_data';

import 'package:path/path.dart' as p;

/// In-memory session store for browser hosts (no filesystem persistence).
class DocumentSessionStore {
  DocumentSessionStore();

  static const maxRecentFiles = 10;
  static const maxRecentSymbols = 12;
  static const defaultAutosaveInterval = Duration(seconds: 60);

  static DocumentSessionStore? _defaultInstance;

  static DocumentSessionStore defaultStore() {
    return _defaultInstance ??= DocumentSessionStore();
  }

  AutosaveSnapshot? _autosave;
  List<RecentDocumentEntry> _recentEntries = const [];
  List<String> _recentSymbolIds = const [];
  Duration _autosaveInterval = defaultAutosaveInterval;

  Future<void> writeAutosave({
    required Uint8List bytes,
    String? sourcePath,
    String format = 'twdoc',
    DateTime? savedAt,
  }) async {
    _autosave = AutosaveSnapshot(
      bytes: bytes,
      savedAt: savedAt ?? DateTime.now(),
      sourcePath: sourcePath,
      format: format,
    );
  }

  Future<AutosaveSnapshot?> readAutosave() async => _autosave;

  Future<void> clearAutosave() async {
    _autosave = null;
  }

  List<RecentDocumentEntry> loadRecentEntries() => _recentEntries;

  List<String> loadRecentPaths() {
    return _recentEntries.map((entry) => entry.path).toList();
  }

  Future<void> saveRecentEntries(List<RecentDocumentEntry> entries) async {
    _recentEntries = entries.take(maxRecentFiles).toList();
  }

  Future<void> saveRecentPaths(List<String> paths) async {
    await saveRecentEntries(
      paths.map((path) => RecentDocumentEntry(path: path)).toList(),
    );
  }

  List<String> loadRecentSymbolIds() => _recentSymbolIds;

  Future<void> saveRecentSymbolIds(List<String> ids) async {
    _recentSymbolIds = ids.take(maxRecentSymbols).toList();
  }

  List<RecentDocumentEntry> bumpRecentEntry(
    List<RecentDocumentEntry> current,
    RecentDocumentEntry entry,
  ) {
    final normalized = p.basename(entry.path);
    final next = [
      RecentDocumentEntry(path: normalized, bookmark: entry.bookmark),
      ...current.where((item) => p.basename(item.path) != normalized),
    ];
    return next.take(maxRecentFiles).toList();
  }

  List<String> bumpRecentPath(List<String> current, String path) {
    final normalized = p.basename(path);
    final next = [
      normalized,
      ...current.where((item) => p.basename(item) != normalized),
    ];
    return next.take(maxRecentFiles).toList();
  }

  Duration loadAutosaveInterval() => _autosaveInterval;

  Future<void> saveAutosaveInterval(Duration interval) async {
    _autosaveInterval = interval;
  }
}

class RecentDocumentEntry {
  const RecentDocumentEntry({required this.path, this.bookmark});

  final String path;
  final String? bookmark;

  factory RecentDocumentEntry.fromJson(Map<String, dynamic> json) {
    return RecentDocumentEntry(
      path: p.basename(json['path'] as String),
      bookmark: json['bookmark'] as String?,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'path': path,
      if (bookmark != null) 'bookmark': bookmark,
    };
  }
}

class AutosaveSnapshot {
  const AutosaveSnapshot({
    required this.bytes,
    required this.savedAt,
    required this.format,
    this.sourcePath,
  });

  final Uint8List bytes;
  final DateTime savedAt;
  final String? sourcePath;
  final String format;
}
