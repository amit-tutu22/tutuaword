import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/editor/display_list.dart';

/// Shared engine connection, display-list cache, and async edit coordination.
class EngineHost extends ChangeNotifier {
  EngineHost({DocumentEngine? engine}) : _engine = engine;

  DocumentEngine? _engine;
  String _documentText = '';
  bool _documentTextStale = false;
  int _displayVersion = 0;
  int _atlasGeneration = 0;
  Uint8List _atlasPixels = Uint8List(0);
  int _atlasWidth = 0;
  int _atlasHeight = 0;
  double _pageWidth = 612;
  double _pageHeight = 792;
  Uint8List _displayListBytes = Uint8List(0);
  int _pageCount = 1;
  int _nativeEditDepth = 0;
  Completer<void>? _editsIdle;
  bool _refreshScheduled = false;
  bool _refreshWantsFull = false;
  final Set<int> _refreshDirtyPages = {};

  final Map<int, Uint8List> pageDisplayLists = {};
  final Map<int, int> pageDisplayVersions = {};

  int get nativeEditDepth => _nativeEditDepth;

  DocumentEngine? get engine => _engine;
  bool get isConnected => _engine != null;

  /// Whole-document text, pulled from the engine only when a caller asks for it
  /// after an edit — keystrokes must not pay for a full text copy over FFI.
  String get documentText {
    if (_documentTextStale) {
      _documentTextStale = false;
      final text = _engine?.fetchDocumentText();
      if (text != null) _documentText = text;
    }
    return _documentText;
  }

  int get displayVersion => _displayVersion;
  int get atlasGeneration => _atlasGeneration;
  Uint8List get atlasPixels => _atlasPixels;
  int get atlasWidth => _atlasWidth;
  int get atlasHeight => _atlasHeight;
  double get pageWidth => _pageWidth;
  double get pageHeight => _pageHeight;
  Uint8List get displayListBytes => _displayListBytes;
  int get pageCount => _pageCount;

  void setEngine(DocumentEngine? engine) {
    _engine = engine;
  }

  void setDocumentText(String text) {
    _documentText = text;
    _documentTextStale = false;
  }

  void setPageCount(int count) {
    _pageCount = count.clamp(1, 9999);
  }

  /// Runs [action] as a tracked in-flight edit; the display refresh it dirties
  /// is coalesced so a burst of edits costs one engine read.
  Future<bool> performNativeEdit(
    Future<bool> Function() action, {
    int? dirtyPage,
    bool full = false,
  }) async {
    _nativeEditDepth++;
    try {
      final ok = await action();
      if (ok) _scheduleRefresh(dirtyPage: dirtyPage, full: full);
      return ok;
    } finally {
      _nativeEditDepth--;
      if (_nativeEditDepth == 0) {
        final idle = _editsIdle;
        _editsIdle = null;
        idle?.complete();
      }
    }
  }

  /// Awaits every in-flight edit — not just the most recent — and then applies
  /// whatever refresh is still coalesced, so callers observe settled state.
  Future<void> ensureLayoutReady() async {
    while (_nativeEditDepth > 0) {
      await (_editsIdle ??= Completer<void>()).future;
    }
    flushPendingRefresh();
  }

  void _scheduleRefresh({int? dirtyPage, bool full = false}) {
    if (full || dirtyPage == null) {
      _refreshWantsFull = true;
    } else {
      _refreshDirtyPages.add(dirtyPage);
    }
    if (_refreshScheduled) return;
    _refreshScheduled = true;
    scheduleMicrotask(flushPendingRefresh);
  }

  /// Applies the coalesced refresh for every edit that completed in this batch.
  void flushPendingRefresh() {
    if (!_refreshScheduled) return;
    _refreshScheduled = false;
    final full = _refreshWantsFull;
    final pages = List<int>.of(_refreshDirtyPages);
    _refreshWantsFull = false;
    _refreshDirtyPages.clear();
    if (full) {
      refreshFromEngine(full: true);
    } else {
      for (final page in pages) {
        refreshFromEngine(dirtyPage: page);
      }
    }
    notifyListeners();
  }

  Uint8List displayListForPage(int page) {
    if (_engine == null) return Uint8List(0);
    final cachedVersion = pageDisplayVersions[page];
    final cached = pageDisplayLists[page];
    if (cached != null && cachedVersion != null) return cached;
    final data = _engine!.fetchPageDisplayList(page);
    if (data == null) return Uint8List(0);
    if (data.bytes.isNotEmpty) {
      pageDisplayLists[page] = data.bytes;
      pageDisplayVersions[page] = data.version;
    }
    return data.bytes;
  }

  int pageDisplayVersion(int page) => pageDisplayVersions[page] ?? _displayVersion;

  void refreshFromEngine({int? dirtyPage, bool full = false}) {
    final engine = _engine;
    if (engine == null) return;
    if (!full && dirtyPage != null && _refreshDirtyPage(engine, dirtyPage)) return;
    final data = engine.fetchDisplayList();
    if (data == null) return;
    if (full) {
      pageDisplayLists.clear();
      pageDisplayVersions.clear();
    } else if (dirtyPage != null) {
      pageDisplayLists.remove(dirtyPage);
      pageDisplayVersions.remove(dirtyPage);
    }
    final versionChanged = data.version != _displayVersion;
    _displayListBytes = data.bytes;
    _displayVersion = data.version;
    if (dirtyPage != null) {
      pageDisplayVersions[dirtyPage] = data.version;
      pageDisplayLists[dirtyPage] = data.bytes;
    }
    _pageWidth = data.pageWidth;
    _pageHeight = data.pageHeight;
    _documentTextStale = true;
    _pageCount = data.pageCount.clamp(1, 9999);
    if (full || versionChanged) _refreshAtlasFromEngine();
  }

  /// Reads back only the edited page instead of the whole document. Returns
  /// false when the engine has no per-page list (the in-memory test engine
  /// returns empty bytes), so the caller can fall back to the full read.
  bool _refreshDirtyPage(DocumentEngine engine, int page) {
    final data = engine.fetchPageDisplayList(page);
    if (data == null || data.bytes.isEmpty) return false;
    final versionChanged = data.version != _displayVersion;
    pageDisplayLists[page] = data.bytes;
    pageDisplayVersions[page] = data.version;
    _displayListBytes = data.bytes;
    _displayVersion = data.version;
    _pageWidth = data.pageWidth;
    _pageHeight = data.pageHeight;
    _documentTextStale = true;
    if (versionChanged) _refreshAtlasFromEngine();
    return true;
  }

  /// The atlas is session-wide and only changes when new glyphs rasterize, so
  /// this is gated on a layout version bump. `fetchAtlas` clones the whole pixel
  /// buffer, so the generation is checked first where the engine can answer it
  /// without the copy.
  void _refreshAtlasFromEngine() {
    final engine = _engine;
    if (engine == null) {
      _atlasGeneration = 0;
      _atlasPixels = Uint8List(0);
      _atlasWidth = 0;
      _atlasHeight = 0;
      return;
    }
    final generation = engine.fetchAtlasGeneration();
    if (generation != null && generation == _atlasGeneration) return;
    final atlas = engine.fetchAtlas();
    if (atlas == null || atlas.generation == _atlasGeneration) return;
    _atlasGeneration = atlas.generation;
    _atlasWidth = atlas.width;
    _atlasHeight = atlas.height;
    _atlasPixels = _atlasPixelsFromWire(atlas.bytes);
  }

  Uint8List _atlasPixelsFromWire(Uint8List wire) {
    if (wire.length < 24) return Uint8List(0);
    final pixelLen =
        ByteData.sublistView(wire, 20, 24).getUint32(0, Endian.little);
    if (wire.length < 24 + pixelLen) return Uint8List(0);
    return Uint8List.sublistView(wire, 24, 24 + pixelLen);
  }

  bool engineHasPaintableDisplayList() {
    if (_displayListBytes.isEmpty) return false;
    final snapshot = DisplayListSnapshot.fromBytes(_displayListBytes);
    return snapshot.hasPaintableGlyphs || snapshot.hasPaintableContent;
  }

  void injectDisplayListForTest(
    Uint8List bytes, {
    int pageCount = 1,
  }) {
    _displayListBytes = bytes;
    _displayVersion++;
    pageDisplayLists.clear();
    pageDisplayVersions.clear();
    _pageCount = pageCount;
    for (var page = 0; page < pageCount; page++) {
      pageDisplayLists[page] = bytes;
      pageDisplayVersions[page] = _displayVersion;
    }
  }

  void clearDisplayCaches() {
    _displayListBytes = Uint8List(0);
    _displayVersion = 0;
    pageDisplayLists.clear();
    pageDisplayVersions.clear();
  }
}
