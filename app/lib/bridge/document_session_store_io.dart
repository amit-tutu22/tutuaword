import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:path/path.dart' as p;

/// Persists autosave drafts, recent file paths, and session settings on disk.
class DocumentSessionStore {
  DocumentSessionStore({required Directory root}) : _root = root;

  static const maxRecentFiles = 10;
  static const maxRecentSymbols = 12;
  static const defaultAutosaveInterval = Duration(seconds: 60);

  final Directory _root;

  static DocumentSessionStore? _defaultInstance;

  static DocumentSessionStore defaultStore() {
    return _defaultInstance ??= DocumentSessionStore(root: _defaultRoot());
  }

  static Directory _defaultRoot() {
    final home = Platform.environment['HOME'];
    if (home != null && home.isNotEmpty) {
      if (Platform.isMacOS || Platform.isLinux) {
        return Directory(p.join(home, '.tutuaword'));
      }
    }
    return Directory(p.join(Directory.systemTemp.path, 'tutuaword'));
  }

  Directory get autosaveDir => Directory(p.join(_root.path, 'autosave'));

  File get _manifestFile => File(p.join(autosaveDir.path, 'manifest.json'));

  File get _draftFile => File(p.join(autosaveDir.path, 'draft.twdoc'));

  File get _recentFile => File(p.join(_root.path, 'recent.json'));

  File get _recentSymbolsFile => File(p.join(_root.path, 'recent_symbols.json'));

  File get _settingsFile => File(p.join(_root.path, 'settings.json'));

  Future<void> writeAutosave({
    required Uint8List bytes,
    String? sourcePath,
    String format = 'twdoc',
    DateTime? savedAt,
  }) async {
    await autosaveDir.create(recursive: true);
    await _draftFile.writeAsBytes(bytes, flush: true);
    final manifest = {
      'savedAt': (savedAt ?? DateTime.now()).toUtc().toIso8601String(),
      if (sourcePath != null) 'sourcePath': sourcePath,
      'format': format,
      'draftFile': 'draft.twdoc',
    };
    await _manifestFile.writeAsString('${jsonEncode(manifest)}\n', flush: true);
  }

  Future<AutosaveSnapshot?> readAutosave() async {
    if (!_manifestFile.existsSync() || !_draftFile.existsSync()) return null;
    try {
      final manifest =
          jsonDecode(await _manifestFile.readAsString()) as Map<String, dynamic>;
      final bytes = await _draftFile.readAsBytes();
      if (bytes.isEmpty) return null;
      return AutosaveSnapshot(
        bytes: bytes,
        savedAt: DateTime.parse(manifest['savedAt'] as String),
        sourcePath: manifest['sourcePath'] as String?,
        format: manifest['format'] as String? ?? 'twdoc',
      );
    } catch (_) {
      return null;
    }
  }

  Future<void> clearAutosave() async {
    if (_draftFile.existsSync()) {
      await _draftFile.delete();
    }
    if (_manifestFile.existsSync()) {
      await _manifestFile.delete();
    }
  }

  List<RecentDocumentEntry> loadRecentEntries() {
    if (!_recentFile.existsSync()) return const [];
    try {
      final decoded = jsonDecode(_recentFile.readAsStringSync()) as List<dynamic>;
      if (decoded.isEmpty) return const [];
      if (decoded.first is String) {
        return decoded
            .whereType<String>()
            .where((path) => path.isNotEmpty && File(path).existsSync())
            .map((path) => RecentDocumentEntry(path: p.normalize(path)))
            .toList();
      }
      return decoded
          .whereType<Map<String, dynamic>>()
          .map(RecentDocumentEntry.fromJson)
          .where((entry) => entry.path.isNotEmpty && File(entry.path).existsSync())
          .toList();
    } catch (_) {
      return const [];
    }
  }

  List<String> loadRecentPaths() {
    return loadRecentEntries().map((entry) => entry.path).toList();
  }

  Future<void> saveRecentEntries(List<RecentDocumentEntry> entries) async {
    await _root.create(recursive: true);
    final trimmed = entries.take(maxRecentFiles).map((e) => e.toJson()).toList();
    await _recentFile.writeAsString('${jsonEncode(trimmed)}\n', flush: true);
  }

  Future<void> saveRecentPaths(List<String> paths) async {
    await saveRecentEntries(
      paths.map((path) => RecentDocumentEntry(path: path)).toList(),
    );
  }

  List<String> loadRecentSymbolIds() {
    if (!_recentSymbolsFile.existsSync()) return const [];
    try {
      final decoded =
          jsonDecode(_recentSymbolsFile.readAsStringSync()) as List<dynamic>;
      return decoded.whereType<String>().where((id) => id.isNotEmpty).toList();
    } catch (_) {
      return const [];
    }
  }

  Future<void> saveRecentSymbolIds(List<String> ids) async {
    await _root.create(recursive: true);
    final trimmed = ids.take(maxRecentSymbols).toList();
    await _recentSymbolsFile.writeAsString('${jsonEncode(trimmed)}\n', flush: true);
  }

  List<RecentDocumentEntry> bumpRecentEntry(
    List<RecentDocumentEntry> current,
    RecentDocumentEntry entry,
  ) {
    final normalized = p.normalize(entry.path);
    RecentDocumentEntry? existing;
    for (final item in current) {
      if (p.normalize(item.path) == normalized) {
        existing = item;
        break;
      }
    }
    final bookmark = entry.bookmark ?? existing?.bookmark;
    final next = [
      RecentDocumentEntry(path: normalized, bookmark: bookmark),
      ...current.where((item) => p.normalize(item.path) != normalized),
    ];
    return next.take(maxRecentFiles).toList();
  }

  List<String> bumpRecentPath(List<String> current, String path) {
    final normalized = p.normalize(path);
    final next = [
      normalized,
      ...current.where((item) => p.normalize(item) != normalized),
    ];
    return next.take(maxRecentFiles).toList();
  }

  Duration loadAutosaveInterval() {
    if (!_settingsFile.existsSync()) return defaultAutosaveInterval;
    try {
      final decoded =
          jsonDecode(_settingsFile.readAsStringSync()) as Map<String, dynamic>;
      final seconds = decoded['autosaveIntervalSeconds'];
      if (seconds is int && seconds > 0) {
        return Duration(seconds: seconds);
      }
    } catch (_) {}
    return defaultAutosaveInterval;
  }

  Future<void> saveAutosaveInterval(Duration interval) async {
    await _root.create(recursive: true);
    await _settingsFile.writeAsString(
      '${jsonEncode({'autosaveIntervalSeconds': interval.inSeconds})}\n',
      flush: true,
    );
  }
}

class RecentDocumentEntry {
  const RecentDocumentEntry({required this.path, this.bookmark});

  final String path;
  final String? bookmark;

  factory RecentDocumentEntry.fromJson(Map<String, dynamic> json) {
    return RecentDocumentEntry(
      path: p.normalize(json['path'] as String),
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
