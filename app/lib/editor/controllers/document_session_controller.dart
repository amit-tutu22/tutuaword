import 'dart:async';
import 'dart:io' if (dart.library.html) 'package:tutuaword/bridge/platform_stub.dart';
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:path/path.dart' as p;
import 'package:tutuaword/bridge/document_io.dart';
import 'package:tutuaword/bridge/document_picker.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/bridge/print_layout_settings.dart';
import 'package:tutuaword/editor/doc_range.dart';
import 'package:tutuaword/bridge/file_bytes.dart';
import 'package:tutuaword/bridge/macos_file_access.dart';
import 'package:tutuaword/bridge/spell_issue.dart';
import 'package:tutuaword/bridge/twdoc_io.dart';
import 'package:tutuaword/editor/autosave_scheduler.dart';
import 'package:tutuaword/editor/controllers/engine_host.dart';
import 'package:tutuaword/editor/controllers/formatting_controller.dart';
import 'package:tutuaword/editor/controllers/selection_controller.dart';
import 'package:tutuaword/editor/controllers/view_controller.dart';
import 'package:tutuaword/editor/document_templates.dart';

typedef SessionNotifyCallback = void Function();

/// Ask the user for a password when opening an encrypted document (F22.S1).
/// Return null to cancel the open.
typedef PasswordPromptCallback = Future<String?> Function({
  String? fileName,
  String? errorMessage,
});

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
    PasswordPromptCallback? passwordPrompt,
  })  : _host = host,
        _selection = selection,
        _formatting = formatting,
        _view = view,
        _passwordPrompt = passwordPrompt,
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
  PasswordPromptCallback? _passwordPrompt;

  AutosaveScheduler? _autosaveScheduler;
  List<RecentDocumentEntry> _recentEntries = const [];
  String? _scopedAccessPath;
  String _statusText = '';
  String? _currentPath;
  bool _documentReadOnly = false;
  bool _encryptionPasswordSet = false;
  DocumentProperties _documentProperties = DocumentProperties.empty;
  String? _infoMessage;
  bool _trackChanges = false;
  List<String> _spellMisspellings = const [];
  List<SpellIssue> _spellIssues = const [];
  List<String> _grammarIssues = const [];
  String? _compareSummary;
  int _editGeneration = 0;
  int _lastAutosavedGeneration = 0;
  bool _autosaveInFlight = false;

  String get statusText => _statusText;
  String? get currentPath => _currentPath;
  bool get documentReadOnly => _documentReadOnly;
  /// True when subsequent DOCX saves will be password-encrypted (F22.S2).
  bool get encryptionPasswordSet => _encryptionPasswordSet;
  DocumentProperties get documentProperties => _documentProperties;
  String? get infoMessage => _infoMessage;
  bool get trackChanges => _trackChanges;
  List<String> get spellMisspellings => _spellMisspellings;
  List<SpellIssue> get spellIssues => _spellIssues;
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

  /// Install/replace the password prompt used for encrypted opens (F22.S1).
  void setPasswordPrompt(PasswordPromptCallback? prompt) {
    _passwordPrompt = prompt;
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
    _spellIssues = const [];
    _infoMessage = null;
    _documentProperties = DocumentProperties.empty;
    _documentReadOnly = false;
    _encryptionPasswordSet = false;
    if (_host.engine != null) {
      _host.engine!.setEncryptionPassword(null);
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
      final picked = await pickDocumentFile(dialogTitle: 'Open document');
      if (picked == null) {
        _statusText = 'Open cancelled';
        notifyListeners();
        return;
      }
      await _openDocumentBytes(picked.bytes, path: picked.path);
    } catch (e) {
      _statusText = 'Open failed: $e';
      notifyListeners();
    }
  }

  /// Open a built-in starter template as an untitled document (F24.S1).
  Future<void> newFromTemplate({
    required Uint8List bytes,
    required String templateTitle,
    required String assetPath,
  }) async {
    _statusText = 'Opening template…';
    notifyListeners();
    try {
      await _openDocumentBytes(
        bytes,
        path: assetPath,
        asUntitled: true,
        statusOverride: 'New from template: $templateTitle',
      );
    } catch (e) {
      _statusText = 'Template open failed: $e';
      notifyListeners();
    }
  }

  /// User templates saved under the session store (F24.S3).
  List<UserTemplateEntry> get userTemplates => _sessionStore.loadUserTemplates();

  Future<Uint8List?> readUserTemplateBytes(String id) =>
      _sessionStore.readUserTemplateBytes(id);

  /// Snapshot the current document as a reusable `.docx` template (F24.S3).
  ///
  /// Does not change [currentPath] or clear dirty state — Save As is unchanged.
  Future<UserTemplateEntry?> saveAsTemplate({
    required String title,
    required String themeName,
  }) async {
    try {
      final bytes = await _serializeDocument(formatExtension: 'docx');
      if (bytes.isEmpty) {
        _statusText = 'Save as template failed: empty document';
        notifyListeners();
        return null;
      }
      final entry = await _sessionStore.saveUserTemplate(
        title: title,
        themeName: themeName,
        bytes: bytes,
      );
      _statusText = 'Saved template: ${entry.title}';
      notifyListeners();
      onSessionChanged();
      return entry;
    } catch (e) {
      _statusText = 'Save as template failed: $e';
      notifyListeners();
      return null;
    }
  }

  Future<void> _openDocumentBytes(
    Uint8List bytes, {
    required String path,
    bool asUntitled = false,
    String? statusOverride,
  }) async {
    var requiresPassword =
        DocumentReader.isPasswordProtectedDocx(bytes, path: path);
    String? password;
    String? promptError;

    if (_host.engine != null) {
      while (true) {
        if (requiresPassword && (password == null || password.isEmpty)) {
          final prompted = await _promptForPassword(
            path: path,
            errorMessage: promptError,
          );
          if (prompted == _PasswordPromptOutcome.cancelled) {
            _statusText = 'Open cancelled';
            notifyListeners();
            return;
          }
          if (prompted == _PasswordPromptOutcome.unavailable) {
            _statusText = 'Password required to open this document';
            notifyListeners();
            return;
          }
          password = prompted.password;
        }

        final code = _host.engine!.openDocumentBytes(
          bytes,
          path: path,
          password: password,
        );
        if (code == 0) {
          _view.reset();
          _host.engine!.setCurrentPageIndex(0);
          _host.refreshFromEngine(full: true);
          _refreshDocumentMetadata();
          _selection.reset();
          _selection.ensureGlyphCaret();
          _formatting.syncFromCaret();
          // Opening with a password retains encryption for subsequent DOCX saves.
          _encryptionPasswordSet =
              requiresPassword && (password?.isNotEmpty ?? false);
          _statusText = statusOverride ??
              (_documentReadOnly ? 'Opened (read-only)' : 'Opened');
          break;
        }

        final err = _host.engine!.getLastError() ?? 'unknown error';
        final lower = err.toLowerCase();
        if (lower.contains('incorrect password')) {
          requiresPassword = true;
          promptError = 'Incorrect password. Try again.';
          password = null;
          if (_passwordPrompt == null) {
            _statusText = 'Incorrect password';
            notifyListeners();
            return;
          }
          continue;
        }
        if (lower.contains('password-protected') ||
            lower.contains('password required')) {
          requiresPassword = true;
          promptError = 'Password required.';
          password = null;
          if (_passwordPrompt == null) {
            _statusText = 'Password required to open this document';
            notifyListeners();
            return;
          }
          continue;
        }
        _statusText = 'Open failed: $err';
        notifyListeners();
        return;
      }
    } else if (requiresPassword) {
      _statusText = 'Password required to open this document';
      notifyListeners();
      return;
    } else {
      _host.setDocumentText(DocumentReader.extractText(bytes, path: path));
      _host.clearDisplayCaches();
      _statusText = statusOverride ?? 'Opened';
    }
    if (asUntitled) {
      _currentPath = null;
      await _sessionStore.clearAutosave();
    } else {
      _currentPath = path;
      await _recordRecentPath(path);
    }
    _syncSavedGeneration();
    notifyListeners();
    onSessionChanged();
    // Chrome often suspends rAF after the file-picker dialog, so post-frame
    // page/atlas loads would sit idle until an unrelated keypress. Force a
    // couple of frames and re-pull display state so the first open paints.
    await _forceOpenPaint();
  }

  Future<void> _forceOpenPaint() async {
    final binding = WidgetsBinding.instance;
    // Prefer timed yields over endOfFrame alone: Chrome can leave rAF suspended
    // after the native file dialog, so awaiting endOfFrame would hang forever.
    for (var i = 0; i < 3; i++) {
      binding.ensureVisualUpdate();
      binding.scheduleFrame();
      await Future<void>.delayed(Duration(milliseconds: 16 * (i + 1)));
      _host.refreshFromEngine(full: true);
      notifyListeners();
      onSessionChanged();
      if (_host.engineHasPaintableDisplayList() && _host.atlasPixels.isNotEmpty) {
        binding.ensureVisualUpdate();
        binding.scheduleFrame();
        // One more frame so DocumentView can finish atlas upload + page load.
        await Future<void>.delayed(const Duration(milliseconds: 32));
        binding.scheduleFrame();
        break;
      }
    }
  }

  Future<_PasswordPromptOutcome> _promptForPassword({
    required String path,
    String? errorMessage,
  }) async {
    final prompt = _passwordPrompt;
    if (prompt == null) {
      return _PasswordPromptOutcome.unavailable;
    }
    final value = await prompt(
      fileName: p.basename(path),
      errorMessage: errorMessage,
    );
    if (value == null) {
      return _PasswordPromptOutcome.cancelled;
    }
    return _PasswordPromptOutcome.submitted(value);
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

  /// Bytes for the OS print dialog — VisualMatch when the engine supports it (F25.S1–S3).
  Uint8List? printPdfBytes([
    PrintLayoutSettings? layout,
    DocRange? selection,
  ]) {
    final engine = _host.engine;
    if (engine == null) return null;
    final forPrint = engine.exportPdfBytesForPrint(layout, selection);
    if (forPrint != null && forPrint.isNotEmpty) return forPrint;
    return engine.exportPdfBytes();
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
      // Single spell pass — cache issues and derive misspelling words.
      final issues = _host.engine!.spellCheckIssues();
      if (issues == null) {
        _statusText = 'Spell check failed';
        notifyListeners();
        return;
      }
      _spellIssues = issues;
      _spellMisspellings = issues.map((issue) => issue.word).toList();
      _statusText = issues.isEmpty
          ? 'No spelling issues found'
          : 'Spell check: ${issues.length} issue(s)';
      if (issues.isNotEmpty) {
        _infoMessage =
            'Spell check found ${issues.length} issue(s): ${_spellMisspellings.take(5).join(", ")}';
      }
    } else {
      _spellIssues = const [];
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
    final spellCount = _spellMisspellings.length;
    await grammarCheckDocument();
    final grammarCount = _grammarIssues.length;
    if (spellCount == 0 && grammarCount == 0) {
      _statusText = 'No spelling or grammar issues found';
    } else {
      // Keep "N issue" phrasing so status consumers can match spelling counts.
      _statusText =
          'Proofing: $spellCount issue(s) spelling, $grammarCount issue(s) grammar';
    }
    notifyListeners();
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

  /// Protect subsequent DOCX saves with [password] (F22.S2).
  bool protectWithPassword(String password) {
    if (_host.engine == null) {
      _statusText = 'Protect unavailable';
      notifyListeners();
      return false;
    }
    if (password.isEmpty) {
      _statusText = 'Password required';
      notifyListeners();
      return false;
    }
    final ok = _host.engine!.setEncryptionPassword(password);
    if (ok) {
      _encryptionPasswordSet = true;
      _statusText = 'Document will be encrypted on DOCX save';
    } else {
      _statusText = 'Protect with password failed';
    }
    notifyListeners();
    onSessionChanged();
    return ok;
  }

  /// Clear encryption password so DOCX saves are plaintext (F22.S2).
  bool removePasswordProtection() {
    if (_host.engine == null) {
      _statusText = 'Remove password unavailable';
      notifyListeners();
      return false;
    }
    final ok = _host.engine!.setEncryptionPassword(null);
    if (ok) {
      _encryptionPasswordSet = false;
      _statusText = 'Password protection removed';
    } else {
      _statusText = 'Remove password failed';
    }
    notifyListeners();
    onSessionChanged();
    return ok;
  }

  /// JSON Document Inspector findings (F22.S3).
  String? fetchDocumentInspectJson() {
    if (_host.engine == null) return '[]';
    return _host.engine!.fetchDocumentInspect();
  }

  /// Remove selected Document Inspector categories (F22.S3).
  bool removeInspectFindings({
    bool comments = false,
    bool metadata = false,
    bool hiddenText = false,
  }) {
    if (_host.engine == null) {
      _statusText = 'Inspect Document unavailable';
      notifyListeners();
      return false;
    }
    if (!comments && !metadata && !hiddenText) {
      return true;
    }
    final ok = _host.engine!.removeInspectFindings(
      comments: comments,
      metadata: metadata,
      hiddenText: hiddenText,
    );
    if (ok) {
      _refreshDocumentMetadata();
      _statusText = 'Document inspected';
      _editGeneration += 1;
    } else {
      _statusText = 'Inspect Document remove failed';
    }
    notifyListeners();
    onSessionChanged();
    return ok;
  }

  /// JSON digital signatures (F22.S4).
  String? fetchDigitalSignaturesJson() {
    if (_host.engine == null) return '[]';
    return _host.engine!.fetchDigitalSignatures();
  }

  /// JSON signature verification results (F22.S4).
  String? verifyDigitalSignaturesJson() {
    if (_host.engine == null) return '[]';
    return _host.engine!.verifyDigitalSignatures();
  }

  /// Sign the document (F22.S4).
  bool signDocument({
    required String name,
    String email = '',
    String? organization,
  }) {
    if (_host.engine == null) {
      _statusText = 'Sign Document unavailable';
      notifyListeners();
      return false;
    }
    final ok = _host.engine!.signDocument(
      name: name,
      email: email,
      organization: organization,
    );
    if (ok) {
      _statusText = 'Document signed by $name';
      _editGeneration += 1;
    } else {
      _statusText = 'Sign Document failed';
    }
    notifyListeners();
    onSessionChanged();
    return ok;
  }

  /// Remove all digital signatures (F22.S4).
  bool clearDigitalSignatures() {
    if (_host.engine == null) {
      _statusText = 'Clear signatures unavailable';
      notifyListeners();
      return false;
    }
    final ok = _host.engine!.clearDigitalSignatures();
    if (ok) {
      _statusText = 'Digital signatures removed';
      _editGeneration += 1;
    } else {
      _statusText = 'Clear signatures failed';
    }
    notifyListeners();
    onSessionChanged();
    return ok;
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

class _PasswordPromptOutcome {
  const _PasswordPromptOutcome._(this.password, this._kind);
  final String? password;
  final int _kind;

  static const unavailable = _PasswordPromptOutcome._(null, 0);
  static const cancelled = _PasswordPromptOutcome._(null, 1);
  static _PasswordPromptOutcome submitted(String password) =>
      _PasswordPromptOutcome._(password, 2);

  @override
  bool operator ==(Object other) =>
      other is _PasswordPromptOutcome &&
      other._kind == _kind &&
      other.password == password;

  @override
  int get hashCode => Object.hash(_kind, password);
}
