import 'dart:async';
import 'dart:io' if (dart.library.html) 'package:tutuaword/bridge/platform_stub.dart';
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as p;
import 'package:tutuaword/bridge/document_io.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/bridge/file_bytes.dart';
import 'package:tutuaword/bridge/macos_file_access.dart';
import 'package:tutuaword/bridge/twdoc_io.dart';
import 'package:tutuaword/editor/autosave_scheduler.dart';
import 'package:tutuaword/editor/controllers/engine_host.dart';
import 'package:tutuaword/editor/controllers/formatting_controller.dart';
import 'package:tutuaword/editor/controllers/selection_controller.dart';
import 'package:tutuaword/editor/controllers/view_controller.dart';

typedef SessionNotifyCallback = void Function();

/// Open/save/autosave/recents/properties and document lifecycle.
class DocumentSessionController extends ChangeNotifier {
  DocumentSessionController({
    required EngineHost host,
    required SelectionController selection,
    required FormattingController formatting,
    required ViewController view,
    required this.onSessionChanged,
    DocumentSessionStore? sessionStore,
    bool enableAutosave = true,
    Duration? autosaveInterval,
  })  : _host = host,
        _selection = selection,
        _formatting = formatting,
        _view = view,
        _sessionStore = sessionStore ?? DocumentSessionStore.defaultStore() {
    _recentEntries = _sessionStore.loadRecentEntries();
    if (enableAutosave) {
      final interval = autosaveInterval ?? _sessionStore.loadAutosaveInterval();
      _autosaveScheduler = AutosaveScheduler(
        interval: interval,
        onTick: performAutosave,
      );
      _autosaveScheduler!.start();
    }
  }

  final EngineHost _host;
  final SelectionController _selection;
  final FormattingController _formatting;
  final ViewController _view;
  final SessionNotifyCallback onSessionChanged;
  final DocumentSessionStore _sessionStore;

  AutosaveScheduler? _autosaveScheduler;
  List<RecentDocumentEntry> _recentEntries = const [];
  String? _scopedAccessPath;
  String _statusText = '';
  String? _currentPath;
  bool _documentReadOnly = false;
  DocumentProperties _documentProperties = DocumentProperties.empty;
  String? _infoMessage;
  bool _trackChanges = false;
  List<String> _spellMisspellings = const [];
  List<String> _grammarIssues = const [];
  String? _compareSummary;
  int _editGeneration = 0;
  int _lastAutosavedGeneration = 0;
  bool _autosaveInFlight = false;

  String get statusText => _statusText;
  String? get currentPath => _currentPath;
  bool get documentReadOnly => _documentReadOnly;
  DocumentProperties get documentProperties => _documentProperties;
  String? get infoMessage => _infoMessage;
  bool get trackChanges => _trackChanges;
  List<String> get spellMisspellings => _spellMisspellings;
  List<String> get grammarIssues => _grammarIssues;
  String? get compareSummary => _compareSummary;
  int get editGeneration => _editGeneration;

  List<String> get recentDocuments =>
      List.unmodifiable(_recentEntries.map((e) => e.path));

  Duration get autosaveInterval =>
      _autosaveScheduler?.interval ?? DocumentSessionStore.defaultAutosaveInterval;

  String get documentTitle {
    if (_currentPath == null) return 'Document1';
    final name = p.basename(_currentPath!);
    final dot = name.lastIndexOf('.');
    return dot == -1 ? name : name.substring(0, dot);
  }

  int get wordCount {
    final text = _host.documentText;
    if (text.trim().isEmpty) return 0;
    return text.trim().split(RegExp(r'\s+')).length;
  }

  void setStatusText(String text) {
    _statusText = text;
    notifyListeners();
    onSessionChanged();
  }

  void clearInfoMessage() {
    _infoMessage = null;
    notifyListeners();
  }

  void markDocumentDirty() {
    _editGeneration++;
  }

  void _syncSavedGeneration() => _lastAutosavedGeneration = _editGeneration;

  void _refreshDocumentMetadata() {
    if (_host.engine == null) {
      _documentProperties = DocumentProperties.empty;
      _documentReadOnly = false;
      return;
    }
    _documentProperties = _host.engine!.fetchDocumentProperties();
    _documentReadOnly = _host.engine!.isDocumentReadOnly();
  }

  void setDocumentReadOnlyForTest(bool readOnly) {
    _documentReadOnly = readOnly;
    notifyListeners();
  }

  Future<void> setAutosaveInterval(Duration interval) async {
    await _sessionStore.saveAutosaveInterval(interval);
    _autosaveScheduler?.setInterval(interval);
    notifyListeners();
  }

  Future<void> performAutosave() async {
    if (_editGeneration == _lastAutosavedGeneration) return;
    while (_host.nativeEditDepth > 0) {
      await Future<void>.delayed(const Duration(milliseconds: 2));
    }
    if (_editGeneration == _lastAutosavedGeneration) return;
    if (_autosaveInFlight) return;
    _autosaveInFlight = true;
    try {
      while (_host.nativeEditDepth > 0) {
        await Future<void>.delayed(const Duration(milliseconds: 2));
      }
      if (_editGeneration == _lastAutosavedGeneration) return;
      final bytes = await _serializeDocument(formatExtension: 'twdoc');
      if (bytes.isEmpty) return;
      await _sessionStore.writeAutosave(
        bytes: bytes,
        sourcePath: _currentPath,
        format: 'twdoc',
      );
      _lastAutosavedGeneration = _editGeneration;
    } catch (_) {
      // Autosave failures should not interrupt editing.
    } finally {
      _autosaveInFlight = false;
    }
  }

  Future<bool> tryRecoverAutosave() async {
    final snapshot = await _sessionStore.readAutosave();
    if (snapshot == null) return false;
    final openPath = _autosaveOpenPath(snapshot);
    if (_host.engine != null &&
        _host.engine!.openDocumentBytes(snapshot.bytes, path: openPath) == 0) {
      _currentPath = snapshot.sourcePath;
      _view.reset();
      _host.engine!.setCurrentPageIndex(0);
      _host.refreshFromEngine(full: true);
      _refreshDocumentMetadata();
      _selection.reset();
      _selection.ensureGlyphCaret();
      _formatting.syncFromCaret();
      _syncSavedGeneration();
      _statusText = 'Recovered unsaved draft';
      _infoMessage = 'Restored draft from ${snapshot.savedAt.toLocal()}';
      notifyListeners();
      onSessionChanged();
      return true;
    }
    _host.setDocumentText(DocumentReader.extractText(snapshot.bytes, path: openPath));
    _host.clearDisplayCaches();
    _currentPath = snapshot.sourcePath;
    _syncSavedGeneration();
    _statusText = 'Recovered unsaved draft';
    notifyListeners();
    onSessionChanged();
    return true;
  }

  Future<void> newDocument() async {
    if (_host.engine != null) {
      if (!_host.engine!.newDocument()) {
        _statusText = 'New document failed';
        notifyListeners();
        return;
      }
      _resetDocumentState();
      await _sessionStore.clearAutosave();
      _syncSavedGeneration();
      _statusText = 'New document';
      notifyListeners();
      onSessionChanged();
      return;
    }
    _resetDocumentState();
    await _sessionStore.clearAutosave();
    _syncSavedGeneration();
    notifyListeners();
    onSessionChanged();
  }

  void _resetDocumentState() {
    _currentPath = null;
    _view.reset();
    _host.setDocumentText('');
    _host.clearDisplayCaches();
    _host.setPageCount(1);
    _selection.reset();
    _spellMisspellings = const [];
    _infoMessage = null;
    _documentProperties = DocumentProperties.empty;
    _documentReadOnly = false;
    if (_host.engine != null) {
      _host.engine!.setCurrentPageIndex(0);
      _host.refreshFromEngine(full: true);
      _refreshDocumentMetadata();
      _selection.ensureGlyphCaret();
      _formatting.syncFromCaret();
    }
  }

  Future<void> openRecentDocument(String path) => openDocumentFromPath(path);

  Future<void> openDocumentFromPath(String path) async {
    _statusText = 'Opening…';
    notifyListeners();
    try {
      final bytes = await _readDocumentBytes(path);
      await _openDocumentBytes(bytes, path: path);
    } catch (e) {
      _statusText = _openFailureMessage(e);
      notifyListeners();
    }
  }

  Future<void> openDocument() async {
    _statusText = 'Opening…';
    notifyListeners();
    try {
      final useInMemoryBytes =
          kIsWeb || (!kIsWeb && (Platform.isAndroid || Platform.isIOS));
      final result = await FilePicker.pickFiles(
        dialogTitle: 'Open document',
        type: FileType.custom,
        allowedExtensions: kSupportedOpenExtensions,
        // 12.x defaults allowMultiple to true; we open one document at a time.
        allowMultiple: false,
        withData: useInMemoryBytes,
      );
      if (result == null || result.files.isEmpty) {
        _statusText = 'Open cancelled';
        notifyListeners();
        return;
      }
      final file = result.files.single;
      if (useInMemoryBytes && file.bytes != null) {
        final path = file.path ?? file.name;
        await _openDocumentBytes(file.bytes!, path: path);
        return;
      }
      final path = file.path;
      if (path == null) {
        _statusText = 'Open failed: no file path (try again)';
        notifyListeners();
        return;
      }
      final bytes = await _readDocumentBytes(path);
      await _openDocumentBytes(bytes, path: path);
    } catch (e) {
      _statusText = 'Open failed: $e';
      notifyListeners();
    }
  }

  Future<void> _openDocumentBytes(Uint8List bytes, {required String path}) async {
    if (DocumentReader.isPasswordProtectedDocx(bytes, path: path)) {
      _statusText = 'Password-protected documents are not supported';
      notifyListeners();
      return;
    }
    if (_host.engine != null) {
      final code = _host.engine!.openDocumentBytes(bytes, path: path);
      if (code == 0) {
        _view.reset();
        _host.engine!.setCurrentPageIndex(0);
        _host.refreshFromEngine(full: true);
        _refreshDocumentMetadata();
        _selection.reset();
        _selection.ensureGlyphCaret();
        _formatting.syncFromCaret();
        _statusText = _documentReadOnly ? 'Opened (read-only)' : 'Opened';
      } else {
        final err = _host.engine!.getLastError();
        _statusText = (err != null && err.toLowerCase().contains('password'))
            ? 'Password-protected documents are not supported'
            : 'Open failed: ${err ?? 'unknown error'}';
        notifyListeners();
        return;
      }
    } else {
      _host.setDocumentText(DocumentReader.extractText(bytes, path: path));
      _host.clearDisplayCaches();
      _statusText = 'Opened';
    }
    _currentPath = path;
    await _recordRecentPath(path);
    _syncSavedGeneration();
    notifyListeners();
    onSessionChanged();
  }

  Future<void> saveDocument() => _saveWithExtension(
        suggestedPath: _currentPath ?? 'document.twdoc',
        defaultExtension: _extensionFromPath(_currentPath) ?? 'twdoc',
        dialogTitle: 'Save document',
      );

  Future<void> saveDocumentAs({required String extension}) {
    final ext = extension.startsWith('.') ? extension.substring(1) : extension;
    return _saveWithExtension(
      suggestedPath: 'document.$ext',
      defaultExtension: ext,
      dialogTitle: 'Save document as',
      forceFormat: ext,
    );
  }

  Future<bool> saveDocumentToPath(String path, {String? formatExtension}) async {
    try {
      final ext = formatExtension ?? _extensionFromPath(path) ?? 'twdoc';
      final outPath = path.endsWith('.$ext') ? path : '$path.$ext';
      final bytes = await _serializeDocument(formatExtension: formatExtension ?? ext);
      if (kIsWeb) {
        await downloadBytes(filename: p.basename(outPath), bytes: bytes);
      } else {
        await writeBytesToPath(outPath, bytes);
      }
      _currentPath = outPath;
      await _recordRecentPath(outPath);
      await _sessionStore.clearAutosave();
      _syncSavedGeneration();
      _statusText = 'Saved';
      notifyListeners();
      onSessionChanged();
      return true;
    } catch (_) {
      _statusText = 'Save failed';
      notifyListeners();
      return false;
    }
  }

  Future<void> exportPdf() async {
    try {
      final bytes = _host.engine?.exportPdfBytes();
      if (bytes == null || bytes.isEmpty) {
        _statusText = 'PDF export failed';
        notifyListeners();
        return;
      }
      if (kIsWeb) {
        await downloadBytes(filename: 'document.pdf', bytes: bytes);
        _statusText = 'PDF exported';
        notifyListeners();
        return;
      }
      final path = await FilePicker.saveFile(
        dialogTitle: 'Export PDF',
        fileName: 'document.pdf',
        type: FileType.custom,
        allowedExtensions: ['pdf'],
        bytes: bytes,
      );
      _statusText = path == null ? 'PDF export cancelled' : 'PDF exported';
      notifyListeners();
    } catch (e) {
      _statusText = 'PDF export failed: $e';
      notifyListeners();
    }
  }

  Future<bool> exportPdfToPath(String path) async {
    try {
      final outPath = path.endsWith('.pdf') ? path : '$path.pdf';
      final bytes = _host.engine?.exportPdfBytes();
      if (bytes == null || bytes.isEmpty) return false;
      if (kIsWeb) {
        await downloadBytes(filename: p.basename(outPath), bytes: bytes);
      } else {
        await writeBytesToPath(outPath, bytes);
      }
      _statusText = 'PDF exported';
      notifyListeners();
      return true;
    } catch (_) {
      return false;
    }
  }

  Future<Uint8List> _serializeDocument({String? formatExtension}) async {
    if (_host.engine != null) {
      if (formatExtension != null) {
        final bytes = _host.engine!.saveDocumentAsBytes(formatExtension) ?? Uint8List(0);
        if (bytes.isNotEmpty) return bytes;
      } else {
        final bytes = _host.engine!.saveDocumentBytes();
        if (bytes != null && bytes.isNotEmpty) return bytes;
      }
    }
    return TwdocWriter.fromText(_host.documentText);
  }

  Future<void> _saveWithExtension({
    required String suggestedPath,
    required String defaultExtension,
    required String dialogTitle,
    String? forceFormat,
  }) async {
    try {
      final ext = forceFormat ?? defaultExtension;
      final bytes = await _serializeDocument(formatExtension: forceFormat);
      if (kIsWeb) {
        final outPath = p.basename(suggestedPath).endsWith('.$ext')
            ? p.basename(suggestedPath)
            : '${p.basename(suggestedPath)}.$ext';
        await downloadBytes(filename: outPath, bytes: bytes);
        _currentPath = outPath;
        await _recordRecentPath(outPath);
        await _sessionStore.clearAutosave();
        _syncSavedGeneration();
        _statusText = 'Saved';
        notifyListeners();
        onSessionChanged();
        return;
      }
      final path = await FilePicker.saveFile(
        dialogTitle: dialogTitle,
        fileName: p.basename(suggestedPath),
        type: FileType.custom,
        allowedExtensions: [defaultExtension],
        bytes: bytes,
      );
      if (path == null) {
        _statusText = 'Save cancelled';
        notifyListeners();
        return;
      }
      final outPath = path.endsWith('.$ext') ? path : '$path.$ext';
      _currentPath = outPath;
      await _recordRecentPath(outPath);
      await _sessionStore.clearAutosave();
      _syncSavedGeneration();
      _statusText = 'Saved';
      notifyListeners();
      onSessionChanged();
    } catch (e) {
      _statusText = 'Save failed: $e';
      notifyListeners();
    }
  }

  /// Autosave bytes are always written in [AutosaveSnapshot.format] (twdoc),
  /// even when [AutosaveSnapshot.sourcePath] still ends in `.docx`.
  String _autosaveOpenPath(AutosaveSnapshot snapshot) {
    final format = snapshot.format.trim().toLowerCase();
    if (format.isNotEmpty) {
      return 'Recovered Draft.$format';
    }
    return snapshot.sourcePath ?? 'Recovered Draft.twdoc';
  }

  String? _extensionFromPath(String? path) {
    if (path == null) return null;
    final dot = path.lastIndexOf('.');
    if (dot == -1) return null;
    return path.substring(dot + 1).toLowerCase();
  }

  Future<void> _persistRecentEntries() =>
      _sessionStore.saveRecentEntries(_recentEntries);

  RecentDocumentEntry _recentEntryForPath(String path) {
    final normalized = p.normalize(path);
    for (final entry in _recentEntries) {
      if (p.normalize(entry.path) == normalized) return entry;
    }
    return RecentDocumentEntry(path: normalized);
  }

  Future<void> _recordRecentPath(String path) async {
    if (kIsWeb) {
      _recentEntries = _sessionStore.bumpRecentEntry(
        _recentEntries,
        RecentDocumentEntry(path: p.basename(path)),
      );
      await _persistRecentEntries();
      notifyListeners();
      return;
    }
    String? bookmark;
    if (Platform.isMacOS) {
      bookmark = await MacOSFileAccess.createBookmark(path);
    }
    _recentEntries = _sessionStore.bumpRecentEntry(
      _recentEntries,
      RecentDocumentEntry(path: path, bookmark: bookmark),
    );
    await _persistRecentEntries();
    notifyListeners();
  }

  Future<void> _releaseScopedAccess() async {
    final previous = _scopedAccessPath;
    _scopedAccessPath = null;
    if (previous != null) await MacOSFileAccess.stopAccess(previous);
  }

  Future<Uint8List> _readDocumentBytes(String path) async {
    if (kIsWeb) {
      throw UnsupportedError('open by path is not supported on web; use Open…');
    }
    await _releaseScopedAccess();
    final entry = _recentEntryForPath(path);
    final ok = await MacOSFileAccess.startAccess(path, bookmark: entry.bookmark);
    if (!ok) throw FileSystemException('Could not access file', path);
    _scopedAccessPath = path;
    return Uint8List.fromList(await File(path).readAsBytes());
  }

  String _openFailureMessage(Object error) {
    final message = error.toString();
    if (Platform.isMacOS &&
        (message.contains('Could not access file') ||
            message.contains('Operation not permitted') ||
            message.contains('Permission denied'))) {
      return 'Open failed: use Open… to select the file again';
    }
    if ((Platform.isAndroid || Platform.isIOS) &&
        (message.contains('Permission denied') ||
            message.contains('Operation not permitted'))) {
      return 'Open failed: grant storage access or use Open… to pick a file';
    }
    return 'Open failed: $error';
  }

  Future<void> undo() async {
    if (_host.engine == null) return;
    final edit = _host.performNativeEdit(() => _host.engine!.undoEditAsync(), full: true);
    if (await edit) {
      _formatting.syncFromCaret();
      markDocumentDirty();
      _statusText = 'Undo';
    }
    notifyListeners();
    onSessionChanged();
  }

  Future<void> redo() async {
    if (_host.engine == null) return;
    final edit = _host.performNativeEdit(() => _host.engine!.redoEditAsync(), full: true);
    if (await edit) {
      _formatting.syncFromCaret();
      markDocumentDirty();
      _statusText = 'Redo';
    }
    notifyListeners();
    onSessionChanged();
  }

  void toggleTrackChanges() {
    _trackChanges = !_trackChanges;
    _host.engine?.setTrackChangesEnabled(_trackChanges);
    _statusText = _trackChanges ? 'Track changes on' : 'Track changes off';
    notifyListeners();
  }

  void acceptAllRevisions() {
    if (_host.engine != null) {
      final ok = _host.engine!.acceptAllRevisions();
      _statusText = ok ? 'Accepted all revisions' : 'Accept revisions failed';
    }
    notifyListeners();
  }

  void rejectAllRevisions() {
    if (_host.engine != null) {
      final ok = _host.engine!.rejectAllRevisions();
      _statusText = ok ? 'Rejected all revisions' : 'Reject revisions failed';
    }
    notifyListeners();
  }

  void acceptRevisionAtCaret() {
    if (_host.engine != null) {
      final ok = _host.engine!.acceptRevisionAtCaret(
        caretRunId: _selection.defaultRunId(),
      );
      _statusText = ok ? 'Accepted revision at caret' : 'No revision at caret';
      if (ok) markDocumentDirty();
    }
    notifyListeners();
    onSessionChanged();
  }

  void rejectRevisionAtCaret() {
    if (_host.engine != null) {
      final ok = _host.engine!.rejectRevisionAtCaret(
        caretRunId: _selection.defaultRunId(),
      );
      _statusText = ok ? 'Rejected revision at caret' : 'No revision at caret';
      if (ok) markDocumentDirty();
    }
    notifyListeners();
    onSessionChanged();
  }

  void gotoNextRevision() {
    final caret = _selection.defaultRunId();
    final next = _host.engine?.adjacentRevisionRunId(caret, forward: true);
    if (next != null) {
      _selection.setCaret(next, 0, page: _selection.caretPage);
      _statusText = 'Next change';
    } else {
      _statusText = 'No tracked changes';
    }
    notifyListeners();
  }

  void gotoPreviousRevision() {
    final caret = _selection.defaultRunId();
    final prev = _host.engine?.adjacentRevisionRunId(caret, forward: false);
    if (prev != null) {
      _selection.setCaret(prev, 0, page: _selection.caretPage);
      _statusText = 'Previous change';
    } else {
      _statusText = 'No tracked changes';
    }
    notifyListeners();
  }

  Future<void> spellCheckDocument() async {
    if (_host.engine != null) {
      // Blocks this isolate — see NativeEngineOps.spellCheckMisspellings.
      final words = _host.engine!.spellCheckMisspellings();
      if (words == null) {
        _statusText = 'Spell check failed';
        notifyListeners();
        return;
      }
      _spellMisspellings = words;
      _statusText = words.isEmpty
          ? 'No spelling issues found'
          : 'Spell check: ${words.length} issue(s)';
      if (words.isNotEmpty) {
        _infoMessage =
            'Spell check found ${words.length} issue(s): ${words.take(5).join(", ")}';
      }
    } else {
      _spellMisspellings = const [];
      _statusText = 'Spell check: no issues';
    }
    notifyListeners();
  }

  Future<void> grammarCheckDocument() async {
    if (_host.engine != null) {
      final issues = _host.engine!.grammarCheckIssues();
      if (issues == null) {
        _statusText = 'Grammar check failed';
        notifyListeners();
        return;
      }
      _grammarIssues = issues;
      _statusText = issues.isEmpty
          ? 'No grammar issues found'
          : 'Grammar check: ${issues.length} issue(s)';
      if (issues.isNotEmpty) {
        _infoMessage =
            'Grammar check found ${issues.length} issue(s): ${issues.take(3).join("; ")}';
      }
    } else {
      _grammarIssues = const [];
      _statusText = 'Grammar check: no issues';
    }
    notifyListeners();
  }

  Future<void> proofDocument() async {
    await spellCheckDocument();
    await grammarCheckDocument();
  }

  void compareWithText(String otherText) {
    if (_host.engine != null) {
      final summary = _host.engine!.compareDocumentText(otherText);
      _compareSummary = summary;
      _statusText = summary == null
          ? 'Compare failed'
          : 'Compare complete ($summary)';
    } else {
      _compareSummary = null;
      _statusText = 'Compare unavailable';
    }
    notifyListeners();
  }

  void toggleRestrictEditing() {
    if (_host.engine != null) {
      final next = !_documentReadOnly;
      final ok = _host.engine!.setReadOnlyEnabled(next);
      if (ok) {
        _documentReadOnly = next;
        _statusText = next ? 'Editing restricted' : 'Editing allowed';
      } else {
        _statusText = 'Restrict editing failed';
      }
    }
    notifyListeners();
    onSessionChanged();
  }

  Future<void> applyEngineStyle(
    Future<bool> Function() action,
    String status, {
    bool full = false,
  }) async {
    if (_host.engine == null) return;
    final edit = _host.performNativeEdit(
      action,
      dirtyPage: full ? null : _selection.caretPage,
      full: full,
    );
    if (await edit) {
      _statusText = status;
      markDocumentDirty();
      _formatting.syncFromCaret();
      notifyListeners();
      onSessionChanged();
    } else {
      final err = _host.engine?.getLastError();
      _statusText = (err != null && err.isNotEmpty) ? err : 'Edit failed';
      notifyListeners();
      onSessionChanged();
    }
  }

  void disposeSession() {
    _autosaveScheduler?.stop();
    unawaited(_releaseScopedAccess());
    if (Platform.isMacOS) unawaited(MacOSFileAccess.stopAllAccess());
  }
}
