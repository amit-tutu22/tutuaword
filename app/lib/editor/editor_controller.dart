import 'dart:async';
import 'dart:convert';
import 'dart:io' if (dart.library.html) 'package:tutuaword/bridge/platform_stub.dart';
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/ui/changes_pane.dart';
import 'package:tutuaword/bridge/accessibility_issue.dart';
import 'package:tutuaword/bridge/bookmark_entry.dart';
import 'package:tutuaword/bridge/comment_thread.dart';
import 'package:tutuaword/bridge/spell_issue.dart';
import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/bridge/document_properties.dart';
import 'package:tutuaword/bridge/engine_loader.dart';
import 'package:tutuaword/bridge/engine_types.dart';
import 'package:tutuaword/bridge/document_print.dart';
import 'package:tutuaword/bridge/document_print_platform.dart';
import 'package:tutuaword/bridge/document_share.dart';
import 'package:tutuaword/bridge/document_share_platform.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/bridge/share_temp.dart';
import 'package:tutuaword/bridge/mock_native_engine.dart';
import 'package:tutuaword/bridge/native_event_router.dart';
import 'package:tutuaword/bridge/print_layout_settings.dart';
import 'package:tutuaword/bridge/outline_entry.dart';
import 'package:tutuaword/bridge/semantic_node.dart';
import 'package:tutuaword/editor/editor_input.dart';
import 'package:tutuaword/editor/controllers/document_session_controller.dart';
import 'package:tutuaword/editor/controllers/engine_host.dart';
import 'package:tutuaword/editor/controllers/find_controller.dart';
import 'package:tutuaword/editor/controllers/formatting_controller.dart';
import 'package:tutuaword/editor/controllers/selection_controller.dart';
import 'package:tutuaword/editor/controllers/view_controller.dart';
import 'package:tutuaword/editor/doc_range.dart';
import 'package:tutuaword/editor/document_edit_zone.dart';
import 'package:tutuaword/bridge/compare_diff.dart';
import 'package:tutuaword/bridge/document_io.dart';
import 'package:tutuaword/bridge/document_picker.dart';
import 'package:tutuaword/ui/compare_results_dialog.dart';
import 'package:tutuaword/editor/change_case.dart';
import 'package:tutuaword/editor/chart_data.dart';
import 'package:tutuaword/editor/equation_omml.dart';
import 'package:tutuaword/editor/display_list.dart';
import 'package:tutuaword/editor/formatting_marks.dart';
import 'package:tutuaword/editor/recent_symbols.dart';
import 'package:tutuaword/editor/image_hit_test.dart';
import 'package:tutuaword/editor/shape_hit_test.dart';
import 'package:tutuaword/editor/key_event_text.dart';
import 'package:tutuaword/editor/text_to_speech.dart';
import 'package:tutuaword/editor/text_to_speech_platform.dart';
import 'package:tutuaword/ui/chart_data_dialog.dart';
import 'package:tutuaword/ui/comment_dialog.dart';
import 'package:tutuaword/ui/cover_page_dialog.dart';
import 'package:tutuaword/ui/equation_dialog.dart';
import 'package:tutuaword/editor/document_templates.dart';
import 'package:tutuaword/editor/document_view_layout.dart';
import 'package:tutuaword/ui/goto_dialog.dart';
import 'package:tutuaword/ui/about_dialog.dart';
import 'package:tutuaword/ui/keyboard_help_dialog.dart';
import 'package:tutuaword/ui/settings_dialog.dart';
import 'package:tutuaword/ui/zoom_dialog.dart';
import 'package:tutuaword/bridge/mail_merge_csv.dart';
import 'package:tutuaword/bridge/ai_client.dart';
import 'package:tutuaword/bridge/ollama_host.dart';
import 'package:tutuaword/bridge/plugin_registry.dart';
import 'package:tutuaword/bridge/ai_generate.dart';
import 'package:tutuaword/bridge/ai_smart_edit.dart';
import 'package:tutuaword/bridge/ai_visual.dart';
import 'package:tutuaword/ui/ai_chat_dialog.dart';
import 'package:tutuaword/ui/ai_generate_dialog.dart';
import 'package:tutuaword/ui/ai_rewrite_dialog.dart';
import 'package:tutuaword/ui/ai_settings_dialog.dart';
import 'package:tutuaword/ui/ai_smart_edit_dialog.dart';
import 'package:tutuaword/ui/ai_translate_dialog.dart';
import 'package:tutuaword/ui/ai_visual_dialog.dart';
import 'package:tutuaword/ui/audience_rewrite_dialog.dart';
import 'package:tutuaword/ui/consistency_checker_dialog.dart';
import 'package:tutuaword/ui/envelopes_labels_dialog.dart';
import 'package:tutuaword/ui/comments_pane.dart';
import 'package:tutuaword/ui/spell_suggestions_dialog.dart';
import 'package:tutuaword/ui/form_field_dialog.dart';
import 'package:tutuaword/ui/hyperlink_dialog.dart';
import 'package:tutuaword/ui/language_dialog.dart';
import 'package:tutuaword/ui/mail_merge_dialogs.dart';
import 'package:tutuaword/ui/plugins_dialog.dart';
import 'package:tutuaword/ui/new_from_template_dialog.dart';
import 'package:tutuaword/ui/save_as_template_dialog.dart';
import 'package:tutuaword/bridge/digital_signature.dart';
import 'package:tutuaword/bridge/document_inspect_finding.dart';
import 'package:tutuaword/bridge/external_link.dart';
import 'package:tutuaword/bridge/proofing_language.dart';
import 'package:tutuaword/bridge/thesaurus.dart';
import 'package:tutuaword/ui/digital_signature_dialog.dart';
import 'package:tutuaword/ui/document_inspector_dialog.dart';
import 'package:tutuaword/ui/print_settings_dialog.dart';
import 'package:tutuaword/ui/protect_password_dialog.dart';
import 'package:tutuaword/ui/symbol_dialog.dart';
import 'package:tutuaword/ui/paragraph_borders_dialog.dart';
import 'package:tutuaword/ui/paragraph_spacing_dialog.dart';
import 'package:tutuaword/ui/page_borders_dialog.dart';
import 'package:tutuaword/ui/page_setup.dart';
import 'package:tutuaword/ui/table_design_dialog.dart';
import 'package:tutuaword/ui/paste_special_dialog.dart';
import 'package:tutuaword/ui/tab_stops_dialog.dart';
import 'package:tutuaword/ui/thesaurus_dialog.dart';

export 'package:tutuaword/bridge/engine_types.dart'
    show CaretGeometry, GlyphSelectionRect;
export 'package:tutuaword/editor/controllers/formatting_controller.dart'
    show LineSpacingMode;
export 'package:tutuaword/editor/doc_range.dart';

/// Writes bytes to a temporary path for the OS share sheet.
typedef ShareTempWriter = Future<String?> Function({
  required String fileName,
  required Uint8List bytes,
});

/// Clipboard payload read from the system pasteboard.
class EditorClipboardPayload {
  const EditorClipboardPayload({this.plainText, this.html, this.docxBytes});

  final String? plainText;
  final String? html;
  final Uint8List? docxBytes;

  bool get hasFormattedContent =>
      (html != null && html!.trim().isNotEmpty) ||
      (docxBytes != null && docxBytes!.isNotEmpty);
}

/// Thin composition of sub-controllers bridging Flutter UI to the Rust engine.
class EditorController extends ChangeNotifier {
  /// Test/document injection constructor — prefer [forTest] in unit tests.
  EditorController({
    DocumentEngine? engine,
    DocumentSessionStore? sessionStore,
    RecentSymbolsStore? recentSymbols,
    TextToSpeechEngine? textToSpeech,
    DocumentPrintHost? printHost,
    DocumentShareHost? shareHost,
    ShareTempWriter? shareTempWriter,
    PasswordPromptCallback? passwordPrompt,
    bool enableAutosave = true,
    Duration? autosaveInterval,
    bool useMockWhenEngineMissing = false,
    AiClient? aiClient,
  })  : _recentSymbols = recentSymbols ?? RecentSymbolsStore.instance,
        _sessionStore = sessionStore,
        _tts = textToSpeech ?? createPlatformTextToSpeech(),
        _printHost = printHost ?? createPlatformPrintHost(),
        _shareHost = shareHost ?? createPlatformShareHost(),
        _shareTempWriter = shareTempWriter ?? writeShareTempFile,
        _host = EngineHost(engine: engine ?? (useMockWhenEngineMissing ? MockDocumentEngine() : loadDocumentEngine())),
        _aiClient = aiClient ??
            AiClient.productionDesktop(
              // Tests / mock engines must not spawn Ollama.
              ollamaHost: (useMockWhenEngineMissing || engine is MockDocumentEngine)
                  ? FakeOllamaHost()
                  : null,
            ) {
    _view = ViewController();
    _selection = SelectionController(
      host: _host,
      onSelectionChanged: () {
        _formatting.syncFromCaret();
        unawaited(_onSelectionChangedFormatPainter());
      },
      onCaretPageChanged: (_) => requestEditorFocus(),
    );
    _formatting = FormattingController(host: _host, selection: _selection);
    _find = FindController(
      host: _host,
      selection: _selection,
      onFindChanged: notifyListeners,
    );
    _session = DocumentSessionController(
      host: _host,
      selection: _selection,
      formatting: _formatting,
      view: _view,
      onSessionChanged: notifyListeners,
      sessionStore: sessionStore,
      enableAutosave: enableAutosave,
      autosaveInterval: autosaveInterval,
      passwordPrompt: passwordPrompt,
    );

    _bootStatus = _host.isConnected
        ? 'Rust engine connected'
        : 'Engine unavailable — build libtw_ffi';

    if (_host.isConnected) {
      _host.refreshFromEngine(full: true);
      _selection.ensureGlyphCaret();
      _formatting.syncFromCaret();
      NativeEventRouter.instance.onUnsolicitedEvent = _onUnsolicitedEngineEvent;
    }

    for (final sub in _subControllers) {
      sub.addListener(notifyListeners);
    }

    if (_sessionStore != null) {
      _recentSymbols.loadIds(_sessionStore!.loadRecentSymbolIds());
    }

    // Desktop ribbon / dialogs keep FocusNodes after click. Letter keys then
    // never reach GlyphEditorSurface even though the caret is painted.
    if (!usesSoftKeyboardGlyphInput) {
      HardwareKeyboard.instance.addHandler(_onDesktopDocumentKey);
    }
  }

  /// [_host] is included so a coalesced display refresh repaints even when the
  /// edit that triggered it already notified optimistically.
  List<ChangeNotifier> get _subControllers =>
      [_host, _view, _selection, _formatting, _find, _session];

  /// In-memory engine for widget/unit tests (R2.4).
  factory EditorController.forTest({
    MockDocumentEngine? engine,
    RecentSymbolsStore? recentSymbols,
    TextToSpeechEngine? textToSpeech,
    DocumentPrintHost? printHost,
    DocumentShareHost? shareHost,
    ShareTempWriter? shareTempWriter,
  }) {
    return EditorController(
      engine: engine ?? MockDocumentEngine(),
      recentSymbols: recentSymbols ?? RecentSymbolsStore(),
      textToSpeech: textToSpeech ?? RecordingTextToSpeech(),
      printHost: printHost ?? RecordingPrintHost(),
      shareHost: shareHost ?? RecordingShareHost(),
      // Avoid real filesystem IO under Flutter's fake-async widget tests.
      shareTempWriter: shareTempWriter ??
          ({required fileName, required bytes}) async =>
              '/tmp/tutuaword_share_test/$fileName',
      enableAutosave: false,
    );
  }

  final EngineHost _host;
  final RecentSymbolsStore _recentSymbols;
  final DocumentSessionStore? _sessionStore;
  final TextToSpeechEngine _tts;
  final DocumentPrintHost _printHost;
  final DocumentShareHost _shareHost;
  final ShareTempWriter _shareTempWriter;
  late final ViewController _view;
  late final SelectionController _selection;
  late final FormattingController _formatting;
  late final FindController _find;
  late final DocumentSessionController _session;
  String _documentThemeName = 'Office';
  MailMergeDataSource? _mailMergeData;
  int _mailMergeRowIndex = 0;
  final PluginRegistry _pluginRegistry = PluginRegistry();
  final AiClient _aiClient;
  String _proofingLanguageId = kDefaultProofingLanguageId;
  DocumentEditZone _editZone = DocumentEditZone.body;
  /// Bumped when a ribbon action needs the page surface to reclaim key focus
  /// (Header/Footer edit, etc.). Desktop ribbon Focusables steal focus on tap.
  int _editorFocusEpoch = 0;
  String? _selectedImageId;
  int? _selectedImagePage;
  Rect? _selectedImageRect;
  String? _selectedDiagramId;
  int? _selectedDiagramPage;
  Rect? _selectedDiagramRect;
  Rect? _previewDiagramRect;
  /// True after entering shape/SmartArt/table cell text until the object is
  /// selected again or the user clicks outside the shape.
  bool _shapeTextEditActive = false;
  String? _shapeTextEditShapeId;
  Rect? _previewImageRect;
  ImageResizeHandle? _activeImageHandle;
  Rect? _resizeStartRect;
  Offset? _moveStartPoint;
  Rect? _moveStartRect;
  double _selectedImageRotation = 0;
  List<AccessibilityIssue> _accessibilityIssues = const [];
  List<TrackedChangeEntry> _trackedChanges = const [];
  bool _accessibilityChecked = false;
  bool _readingAloud = false;

  /// Serializes glyph mutations so held Enter/Backspace cannot overlap splits.
  Future<void> _glyphMutationTail = Future<void>.value();

  String _bootStatus = '';
  bool _disposed = false;

  // ── Sub-controller accessors (R2.4 decomposition) ─────────────────────────
  ViewController get view => _view;
  SelectionController get selectionController => _selection;
  FormattingController get formattingController => _formatting;
  DocumentSessionController get sessionController => _session;
  DocumentEditZone get editZone => _editZone;
  int get editorFocusEpoch => _editorFocusEpoch;

  /// Return keyboard focus to the glyph editor after a ribbon / dialog action.
  void requestEditorFocus() {
    _editorFocusEpoch++;
    focusGlyphInput();
    notifyListeners();
  }

  /// Debug label on every [GlyphEditorSurface] [FocusNode]. Used to skip the
  /// desktop hardware handler when the page already owns the key stream.
  static const glyphEditorFocusLabel = 'GlyphEditorSurface';

  /// Same as [requestEditorFocus], then again after the next two frames so a
  /// popped dialog cannot restore ribbon focus on top of the caret.
  void requestEditorFocusAfterOverlay() {
    requestEditorFocus();
    void again() {
      if (_disposed) return;
      requestEditorFocus();
    }

    WidgetsBinding.instance.addPostFrameCallback((_) {
      again();
      WidgetsBinding.instance.addPostFrameCallback((_) => again());
    });
  }

  /// Desktop: insert printable keys even when a ribbon button still has focus.
  ///
  /// Space/Enter on a [FocusableActionDetector] still activate the control.
  /// Text fields (Find, dialogs) keep their keys. Web/mobile use the hidden
  /// [TextField] instead. When the glyph surface already has focus it handles
  /// keys itself so this path must not double-insert.
  bool _onDesktopDocumentKey(KeyEvent event) {
    if (_disposed) return false;
    if (event is! KeyDownEvent && event is! KeyRepeatEvent) return false;
    if (_glyphEditorHasPrimaryFocus()) return false;
    if (_isTypingIntoTextField()) return false;
    if (_isRibbonActivateKey(event)) return false;

    final editing = EditorInputEvent.fromKeyEvent(event);
    if (editing != null) {
      if (editing.kind == EditorInputKind.character) {
        final char = editing.character ?? printableCharacterFromKeyEvent(event);
        if (char == null ||
            char.isEmpty ||
            HardwareKeyboard.instance.isControlPressed ||
            HardwareKeyboard.instance.isMetaPressed) {
          return false;
        }
        unawaited(handleEditorInput(EditorInputEvent.character(char)));
        return true;
      }
      unawaited(handleEditorInput(editing));
      return true;
    }

    final char = printableCharacterFromKeyEvent(event);
    if (char == null ||
        char.isEmpty ||
        HardwareKeyboard.instance.isControlPressed ||
        HardwareKeyboard.instance.isMetaPressed) {
      return false;
    }
    unawaited(handleEditorInput(EditorInputEvent.character(char)));
    return true;
  }

  bool _glyphEditorHasPrimaryFocus() {
    return FocusManager.instance.primaryFocus?.debugLabel ==
        glyphEditorFocusLabel;
  }

  bool _isTypingIntoTextField() {
    final focused = FocusManager.instance.primaryFocus?.context;
    if (focused == null) return false;
    return focused.widget is EditableText ||
        focused.findAncestorWidgetOfExactType<EditableText>() != null ||
        focused.findAncestorWidgetOfExactType<TextField>() != null;
  }

  bool _isRibbonActivateKey(KeyEvent event) {
    final key = event.logicalKey;
    if (key != LogicalKeyboardKey.space &&
        key != LogicalKeyboardKey.enter &&
        key != LogicalKeyboardKey.numpadEnter) {
      return false;
    }
    final focused = FocusManager.instance.primaryFocus?.context;
    if (focused == null) return false;
    return focused.findAncestorWidgetOfExactType<FocusableActionDetector>() !=
        null;
  }

  // ── Engine / rendering ────────────────────────────────────────────────────
  bool get isEngineConnected => _host.isConnected;
  bool get usesGlyphRendering => _host.isConnected;

  /// Focus target for the hidden glyph [TextField] (web + mobile soft keyboard).
  final FocusNode webGlyphFocusNode = FocusNode();

  /// When the DOM key listener handles Enter/Tab/arrows, skip the next
  /// [TextField.onChanged] so the same keystroke cannot fire twice.
  bool _webSkipNextFieldChange = false;

  void markWebSpecialKeyConsumed() {
    _webSkipNextFieldChange = true;
  }

  bool consumeWebSkipNextFieldChange() {
    if (!_webSkipNextFieldChange) return false;
    _webSkipNextFieldChange = false;
    return true;
  }

  void focusGlyphInput() {
    if (!usesSoftKeyboardGlyphInput) return;
    if (!webGlyphFocusNode.canRequestFocus) return;
    // Already focused — re-requesting on every tap makes iOS recreate the
    // keyboard keyplane (TUIKeyplane / UIKeyboardImpl constraint spam).
    if (webGlyphFocusNode.hasFocus) return;
    webGlyphFocusNode.requestFocus();
    // Web sometimes drops DOM focus after rebuild; retry once.
    if (kIsWeb) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (webGlyphFocusNode.canRequestFocus && !webGlyphFocusNode.hasFocus) {
          webGlyphFocusNode.requestFocus();
        }
      });
    }
  }

  /// Hidden TextField overlay for web and mobile soft-keyboard input.
  static bool get usesSoftKeyboardGlyphInput {
    if (kIsWeb) return true;
    return Platform.isIOS || Platform.isAndroid;
  }

  bool get preferTextRendering => false;
  Uint8List get displayListBytes => _host.displayListBytes;
  double get pageWidth => _host.pageWidth;
  double get pageHeight => _host.pageHeight;
  double get marginTop => _host.marginTop;
  double get marginBottom => _host.marginBottom;
  double get marginLeft => _host.marginLeft;
  double get marginRight => _host.marginRight;
  bool get isLandscape => pageWidth > pageHeight;
  String get marginPresetName =>
      PageSetupPresets.matchingMarginPreset(_currentSectionFormat()) ?? 'Normal';
  String get pageSizePresetName =>
      PageSetupPresets.matchingPageSizePreset(_currentSectionFormat()) ?? 'Letter';
  String get columnCountLabel =>
      PageSetupPresets.columnCountLabel(_currentSectionFormat());
  Color? get pageColor => PageSetupPresets.pageColor(_currentSectionFormat());
  String? get watermarkText => PageSetupPresets.watermarkText(_currentSectionFormat());
  bool get hasPageBorders =>
      PageSetupPresets.hasPageBorders(_currentSectionFormat());
  double? get pageBorderWidth =>
      PageSetupPresets.pageBorderWidth(_currentSectionFormat());
  Color? get pageBorderColor =>
      PageSetupPresets.pageBorderColor(_currentSectionFormat());
  bool get lineNumbersEnabled =>
      PageSetupPresets.lineNumbersEnabled(_currentSectionFormat());
  bool get differentFirstPage =>
      _currentSectionFormat()['different_first_page'] as bool? ?? false;
  bool get evenAndOddHeaders =>
      _host.engine?.fetchEvenAndOddHeadersEnabled() ?? false;
  bool get headerFooterLinked =>
      _host.engine?.fetchHeaderFooterLinked(
        caretRunId: _selection.defaultRunId(),
        isHeader: _editZone == DocumentEditZone.header,
        pageIndex: _selection.caretPage,
      ) ??
      false;
  bool get canLinkHeaderFooter => _editZone != DocumentEditZone.body;
  int get displayVersion => _host.displayVersion;
  int get atlasGeneration => _host.atlasGeneration;
  Uint8List get atlasPixels => _host.atlasPixels;
  int get atlasWidth => _host.atlasWidth;
  int get atlasHeight => _host.atlasHeight;
  int get pageCount => _host.pageCount;
  Uint8List displayListForPage(int page) => _host.displayListForPage(page);
  int pageDisplayVersion(int page) => _host.pageDisplayVersion(page);

  // ── Session ───────────────────────────────────────────────────────────────
  String get statusText {
    final base = _session.statusText.isNotEmpty ? _session.statusText : _bootStatus;
    final path = _session.currentPath == null
        ? ''
        : ' · ${_session.currentPath!.split(Platform.pathSeparator).last}';
    final preview = _view.printPreview ? ' · Print preview' : '';
    return '$base$path$preview · ${_host.documentText.length} chars · page ${_view.currentPage + 1}/$pageCount · v$displayVersion';
  }

  String get documentText => _host.documentText;
  String? get currentPath => _session.currentPath;
  bool get documentReadOnly => _session.documentReadOnly;
  DocumentProperties get documentProperties => _session.documentProperties;
  String? get infoMessage => _session.infoMessage;
  bool get trackChanges => _session.trackChanges;
  List<String> get spellMisspellings => _session.spellMisspellings;
  List<String> get grammarIssues => _session.grammarIssues;
  String? get compareSummary => _session.compareSummary;
  bool get findPaneVisible => _find.paneVisible;
  String get findQuery => _find.query;
  String get findReplaceText => _find.replaceText;
  bool get findMatchCase => _find.matchCase;
  bool get findUseRegex => _find.useRegex;
  bool get findUseWildcards => _find.useWildcards;
  bool get findBold => _find.findBold;
  String get findStyleName => _find.findStyleName;
  String get findStatusText => _find.statusText;
  void openFindPane({String? initialQuery}) => _find.openPane(initialQuery: initialQuery);
  void closeFindPane() => _find.closePane();
  void setFindQuery(String value) => _find.setQuery(value);
  void setFindReplaceText(String value) => _find.setReplaceText(value);
  Future<int?> replaceAll() => _find.replaceAll();
  void toggleFindMatchCase() => _find.toggleMatchCase();
  void toggleFindUseRegex() => _find.toggleUseRegex();
  void toggleFindUseWildcards() => _find.toggleUseWildcards();
  void toggleFindBold() => _find.toggleFindBold();
  void setFindStyleName(String value) => _find.setFindStyleName(value);
  void findNext() => _find.findNext();
  void findPrevious() => _find.findPrevious();
  List<String> get recentDocuments => _session.recentDocuments;
  RecentSymbolsStore get recentSymbols => _recentSymbols;
  Duration get autosaveInterval => _session.autosaveInterval;
  String get documentTitle => _session.documentTitle;
  int get wordCount => _session.wordCount;

  @visibleForTesting
  int get nativeEditDepth => _host.nativeEditDepth;

  @visibleForTesting
  Future<void> ensureLayoutReady() => _host.ensureLayoutReady();

  void _onUnsolicitedEngineEvent(int eventType, int _) {
    if (_disposed) return;
    if (eventType != NativeEventTypes.displayListReady) return;
    final pageBefore = caretPage;
    // Background reflow — refresh only the visible page to avoid full-doc churn.
    _host.refreshFromEngine(dirtyPage: currentPage);
    _selection.syncCaretGeometry();
    if (caretPage != pageBefore) {
      ensureCaretVisible(notify: false);
    }
    notifyListeners();
  }

  // ── View ──────────────────────────────────────────────────────────────────
  int get currentPage => _view.currentPage;

  /// Page number for the status bar: caret page while editing, else scroll page.
  int get statusPage => caretRunId != null ? caretPage : currentPage;
  bool get printPreview => _view.printPreview;
  DocumentViewLayout get viewLayout => _view.layout;
  bool get isReadMode => _view.isReadMode;
  bool get isWebLayout => _view.isWebLayout;
  bool get splitView => _view.splitView;
  int get pageColumns => _view.pageColumns;
  double get zoom => _view.zoom;
  bool get showRuler => _view.showRuler;
  bool get showFormattingMarks => _view.showFormattingMarks;
  bool get showNavigationPane => _view.showNavigationPane;
  bool get showStyleInspector => _view.showStyleInspector;
  bool get showAccessibilityChecker => _view.showAccessibilityChecker;
  bool get showChangesPane => _view.showChangesPane;
  List<AccessibilityIssue> get accessibilityIssues => _accessibilityIssues;
  List<TrackedChangeEntry> get trackedChanges => _trackedChanges;
  bool get accessibilityChecked => _accessibilityChecked;

  /// Status-bar summary for the last accessibility check (F21.S4).
  String get accessibilityStatusLabel {
    if (!_accessibilityChecked) return 'Accessibility: Not checked';
    if (_accessibilityIssues.isEmpty) return 'Accessibility: Good to go';
    final errors =
        _accessibilityIssues.where((i) => i.isError).length;
    final warnings =
        _accessibilityIssues.where((i) => i.isWarning).length;
    if (errors > 0 && warnings > 0) {
      return 'Accessibility: $errors error(s), $warnings warning(s)';
    }
    if (errors > 0) return 'Accessibility: $errors error(s)';
    return 'Accessibility: $warnings warning(s)';
  }
  String get styleInspectorSummary => _formatting.styleInspectorSummary;

  void setCurrentPage(int page) {
    _selectPage(page);
    _host.refreshFromEngine(dirtyPage: _view.currentPage);
    notifyListeners();
  }

  /// Select [page] without a full display-list refresh (F19.S1).
  ///
  /// Prefer this (or [jumpToPage]) from the navigation strip so mock/injected
  /// multi-page state is not overwritten by a single-page engine snapshot.
  void selectPage(int page) {
    _selectPage(page);
    notifyListeners();
  }

  /// Jump to [page] and request the document canvas to scroll there (F19.S1).
  void jumpToPage(int page) {
    _selectPage(page);
    _view.requestScrollToPage(_view.currentPage);
    notifyListeners();
  }

  void _selectPage(int page) {
    final before = _view.currentPage;
    _view.setCurrentPage(page, pageCount);
    if (_view.currentPage == before) return;
    _host.engine?.setCurrentPageIndex(_view.currentPage);
  }

  void setVisiblePage(int page) => _view.setVisiblePage(page, pageCount);
  void togglePrintPreview() {
    _view.togglePrintPreview();
    _session.setStatusText(_view.printPreview ? 'Print preview' : 'Print layout');
    notifyListeners();
  }

  void setPrintLayout() {
    _view.setPrintLayout();
    _session.setStatusText('Print layout');
    notifyListeners();
  }

  void setReadMode() {
    _view.setReadMode();
    _session.setStatusText('Read mode');
    notifyListeners();
  }

  void setWebLayout() {
    _view.setWebLayout();
    _session.setStatusText('Web layout');
    notifyListeners();
  }

  void setPrintPreviewMode() {
    _view.setPrintPreviewMode();
    _session.setStatusText('Print preview');
    notifyListeners();
  }

  void setZoom(double value) => _view.setZoom(value);
  void zoomIn() => _view.zoomIn();
  void zoomOut() => _view.zoomOut();

  Future<void> openZoomDialog(BuildContext context) async {
    final next = await ZoomDialog.show(context, currentZoom: zoom);
    if (next == null) return;
    setZoom(next);
    _session.setStatusText('Zoom ${(next * 100).round()}%');
    notifyListeners();
  }

  Future<void> openAboutDialog(BuildContext context) =>
      TutuawordAboutDialog.show(context);

  Future<void> openKeyboardHelpDialog(BuildContext context) =>
      KeyboardHelpDialog.show(context);

  Future<void> openSettingsDialog(BuildContext context) =>
      AppSettingsDialog.show(context);

  void zoomToOnePage() {
    _view.zoomToOnePage(pageWidth: pageWidth, pageHeight: pageHeight);
    _session.setStatusText('One page · ${(zoom * 100).round()}%');
    notifyListeners();
  }

  void zoomToMultiplePages() {
    _view.zoomToMultiplePages(pageWidth: pageWidth, pageHeight: pageHeight);
    _session.setStatusText('Multiple pages · ${(zoom * 100).round()}%');
    notifyListeners();
  }

  void reportViewportSize(Size size) {
    _view.reportViewport(width: size.width, height: size.height);
  }

  void toggleSplitView() {
    _view.toggleSplitView();
    _session.setStatusText(_view.splitView ? 'Split view' : 'Split view closed');
    notifyListeners();
  }

  void arrangeAllViews() {
    _view.setSplitView(true);
    _session.setStatusText('Arranged views');
    notifyListeners();
  }

  void markNewWindowOpened() {
    _session.setStatusText('New window');
    notifyListeners();
  }

  void markNewWindowClosed() {
    _session.setStatusText('Window closed');
    notifyListeners();
  }

  void toggleRuler() => _view.toggleRuler();

  void toggleFormattingMarks() {
    _view.toggleFormattingMarks();
    _session.setStatusText(
      _view.showFormattingMarks
          ? 'Formatting marks shown'
          : 'Formatting marks hidden',
    );
    notifyListeners();
  }

  /// Non-printing mark positions for [pageIndex] from display-list payload.
  List<FormattingMark> formattingMarksForPage(int pageIndex, DisplayListSnapshot? snapshot) {
    if (!_view.showFormattingMarks || snapshot == null) {
      return const [];
    }
    return formattingMarksFromSnapshot(snapshot);
  }
  void toggleNavigationPane() => _view.toggleNavigationPane();
  void showNavigationOutline() {
    _view.showNavigationOutline();
    _session.setStatusText('Outline');
  }

  void toggleStyleInspector() => _view.toggleStyleInspector();
  void hideAccessibilityChecker() => _view.hideAccessibilityCheckerPane();

  /// Show the tracked-changes list pane and refresh from engine (F17.S2).
  Future<void> showChangesPanePanel() async {
    await refreshRevisions();
    _view.showChangesPanePanel();
    notifyListeners();
  }

  void hideChangesPane() => _view.hideChangesPanePanel();

  /// Reload tracked-change entries from the engine.
  Future<void> refreshRevisions() async {
    final json = _host.engine?.fetchRevisions();
    if (json == null || json.isEmpty) {
      _trackedChanges = const [];
      notifyListeners();
      return;
    }
    final decoded = jsonDecode(json);
    if (decoded is! List) {
      _trackedChanges = const [];
    } else {
      _trackedChanges = decoded
          .whereType<Map>()
          .map((e) => TrackedChangeEntry.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    }
    notifyListeners();
  }

  /// Drop a change from the local list after accept/reject (avoids a full refetch).
  void removeTrackedChangeLocally(String runId) {
    if (runId.isEmpty) return;
    final next = _trackedChanges.where((e) => e.runId != runId).toList();
    if (next.length == _trackedChanges.length) return;
    _trackedChanges = next;
    notifyListeners();
  }

  /// Move caret to a revision-marked run (Changes pane).
  void focusRevision(String runId) {
    if (runId.isEmpty) return;
    clearImageSelection();
    clearDiagramSelection();
    _selection.setCaret(runId, 0);
    _session.setStatusText('Tracked change');
    notifyListeners();
  }

  /// Run F21.S4 accessibility rules and show the results pane.
  void checkAccessibility() {
    final json = _host.engine?.fetchAccessibilityIssues();
    if (json == null) {
      _accessibilityIssues = const [];
      _accessibilityChecked = true;
      _session.setStatusText('Accessibility check failed');
      _view.showAccessibilityCheckerPane();
      notifyListeners();
      return;
    }
    final decoded = jsonDecode(json);
    if (decoded is! List) {
      _accessibilityIssues = const [];
    } else {
      _accessibilityIssues = decoded
          .whereType<Map>()
          .map((e) => AccessibilityIssue.fromJson(Map<String, dynamic>.from(e)))
          .toList();
    }
    _accessibilityChecked = true;
    _session.setStatusText(accessibilityStatusLabel);
    _view.showAccessibilityCheckerPane();
    notifyListeners();
  }

  /// Jump to an accessibility finding (caret / image selection).
  void focusAccessibilityIssue(AccessibilityIssue issue) {
    if (issue.rule == 'missing_alt' && issue.nodeId.isNotEmpty) {
      clearDiagramSelection();
      _selectedImageId = issue.nodeId;
      _selectedImagePage = _view.currentPage;
      _selectedImageRect = null;
      _previewImageRect = null;
    } else if (issue.runId != null && issue.runId!.isNotEmpty) {
      clearImageSelection();
      _selection.setCaret(issue.runId!, 0);
    }
    _session.setStatusText(issue.message);
    notifyListeners();
  }

  List<DocumentOutlineEntry> get documentOutline {
    final json = _host.engine?.fetchDocumentOutline();
    if (json == null || json.isEmpty) return const [];
    final decoded = jsonDecode(json);
    if (decoded is! List) return const [];
    return decoded
        .whereType<Map>()
        .map((entry) => DocumentOutlineEntry.fromJson(Map<String, dynamic>.from(entry)))
        .toList();
  }

  /// Jump to a heading / outline entry and scroll the canvas (F19.S2).
  void jumpToOutlineEntry(DocumentOutlineEntry entry) {
    _selection.setCaret(entry.runId, 0, page: entry.page);
    _selectPage(entry.page);
    _view.requestScrollToPage(entry.page);
    _session.setStatusText('Outline: ${entry.text}');
    notifyListeners();
  }

  List<DocumentBookmarkEntry> get documentBookmarks {
    final json = _host.engine?.fetchBookmarks();
    if (json == null || json.isEmpty) return const [];
    final decoded = jsonDecode(json);
    if (decoded is! List) return const [];
    return decoded
        .whereType<Map>()
        .map((entry) => DocumentBookmarkEntry.fromJson(Map<String, dynamic>.from(entry)))
        .toList();
  }

  /// Parallel accessibility tree from the engine (F21.S1).
  List<SemanticDocumentNode> get semanticDocumentTree {
    final json = _host.engine?.fetchSemanticTree();
    if (json == null || json.isEmpty) return const [];
    final decoded = jsonDecode(json);
    if (decoded is! List) return const [];
    return decoded
        .whereType<Map>()
        .map((entry) => SemanticDocumentNode.fromJson(Map<String, dynamic>.from(entry)))
        .toList();
  }

  /// Jump to a bookmark and scroll the canvas (F19.S4).
  void jumpToBookmark(DocumentBookmarkEntry entry) {
    _selection.setCaret(entry.runId, 0, page: entry.page);
    _selectPage(entry.page);
    _view.requestScrollToPage(entry.page);
    _session.setStatusText('Bookmark: ${entry.name}');
    notifyListeners();
  }

  /// Follow a hyperlink at the caret. Internal anchors always navigate;
  /// external URLs open only when [allowExternal] is true (mobile tap or
  /// Ctrl/Cmd+click). When [context] is set, the user must confirm before
  /// leaving the app. Only http(s) / mailto / tel schemes are allowed.
  Future<bool> tryFollowHyperlink({
    required bool allowExternal,
    BuildContext? context,
  }) async {
    if (!_host.isConnected) return false;
    final runId = _selection.defaultRunId();
    if (runId == null) return false;
    final json = _host.engine?.fetchHyperlinkAt(runId);
    if (json == null || json.isEmpty) return false;
    final decoded = jsonDecode(json);
    if (decoded is! Map) return false;
    final map = Map<String, dynamic>.from(decoded);
    final url = map['url'] as String? ?? '';
    final rawAnchor = map['anchor'] as String?;
    final text = map['text'] as String? ?? '';
    var bookmarkName = (rawAnchor != null && rawAnchor.isNotEmpty)
        ? rawAnchor
        : (url.startsWith('#') ? url.substring(1) : '');
    if (bookmarkName.startsWith('#')) {
      bookmarkName = bookmarkName.substring(1);
    }
    if (bookmarkName.isNotEmpty) {
      return _jumpToHyperlinkAnchor(bookmarkName, text);
    }
    if (!allowExternal || url.isEmpty || url.startsWith('r:id:')) {
      return false;
    }
    final uri = Uri.tryParse(url);
    if (uri == null || !uri.hasScheme) return false;
    if (!isAllowedDocumentLinkUri(uri)) {
      _session.setStatusText('Blocked unsafe link (${uri.scheme})');
      notifyListeners();
      return false;
    }
    if (context != null && context.mounted) {
      final ok = await confirmOpenDocumentLink(context, uri);
      if (!ok) return false;
    }
    final opened = uri.scheme.toLowerCase() == 'mailto'
        ? await openEmailUri(uri)
        : await openExternalUri(uri);
    if (opened) {
      _session.setStatusText('Opened link');
      notifyListeners();
    }
    return opened;
  }

  bool _jumpToHyperlinkAnchor(String name, String linkText) {
    final lower = name.toLowerCase();
    for (final entry in documentBookmarks) {
      if (entry.name.toLowerCase() == lower) {
        jumpToBookmark(entry);
        return true;
      }
    }
    final needle = linkText.trim().toLowerCase();
    if (needle.isNotEmpty) {
      for (final heading in documentOutline) {
        if (heading.text.trim().toLowerCase() == needle) {
          jumpToOutlineEntry(heading);
          return true;
        }
      }
    }
    return false;
  }

  /// Open Go To (page / bookmark / heading) and navigate (F19.S4).
  Future<void> openGoToDialog(BuildContext context) async {
    final result = await GoToDialog.show(
      context,
      pageCount: pageCount,
      currentPage: currentPage,
      bookmarks: documentBookmarks,
      headings: documentOutline,
    );
    if (result == null) return;
    switch (result) {
      case GoToPageResult(:final pageIndex):
        jumpToPage(pageIndex);
        _session.setStatusText('Go To page ${pageIndex + 1}');
      case GoToBookmarkResult(:final entry):
        jumpToBookmark(entry);
      case GoToHeadingResult(:final entry):
        jumpToOutlineEntry(entry);
    }
  }

  bool isPageEditable(int pageIndex) {
    if (_session.documentReadOnly ||
        _view.printPreview ||
        _view.isReadMode) {
      return false;
    }
    return _host.isConnected;
  }

  // ── Selection ─────────────────────────────────────────────────────────────
  CaretGeometry? get caretGeometry => _selection.caretGeometry;
  int get caretPage => _selection.caretPage;
  List<GlyphSelectionRect> get selectionRects => _selection.selectionRects;
  List<GlyphSelectionRect> selectionRectsForPage(int pageIndex) =>
      _selection.selectionRectsForPage(pageIndex);
  bool get hasGlyphSelection => _selection.hasGlyphSelection;
  DocRange? get selection => _selection.selection;
  String? get caretRunId => _selection.caretRunId;
  int get caretOffset => _selection.caretOffset;

  void ensureGlyphCaret() => _selection.ensureGlyphCaret();
  void hitTestAt(int pageIndex, double x, double y) => _selection.hitTestAt(pageIndex, x, y);
  void moveGlyphCaretByArrow(LogicalKeyboardKey key, {bool? extend}) {
    if (hasSelectedDiagram) {
      // Enter cell/shape text, then honor the same arrow so the caret actually
      // moves instead of only clearing object selection.
      unawaited(() async {
        final entered = await enterSelectedShapeTextEdit();
        if (!entered || _disposed) return;
        _selection.moveGlyphCaretByArrow(key, extend: extend);
        ensureCaretVisible(notify: false);
      }());
      return;
    }
    _selection.moveGlyphCaretByArrow(key, extend: extend);
    // Selection already notified this frame; queue without a second notify.
    ensureCaretVisible(notify: false);
  }

  void extendGlyphSelectionTo(int pageIndex, double x, double y) =>
      _selection.extendGlyphSelectionTo(pageIndex, x, y);

  void moveGlyphCaretToLineEdge({required bool toEnd, bool extend = false}) {
    _selection.moveGlyphCaretToLineEdge(toEnd: toEnd, extend: extend);
    ensureCaretVisible(notify: false);
  }

  void moveGlyphCaretToDocumentEdge({required bool toEnd, bool extend = false}) {
    _selection.moveGlyphCaretToDocumentEdge(toEnd: toEnd, extend: extend);
    ensureCaretVisible(notify: false);
  }

  void moveGlyphCaretByPage({required int direction, bool extend = false}) {
    _selection.moveGlyphCaretByPage(direction: direction, extend: extend);
    ensureCaretVisible(notify: false);
  }

  void moveGlyphCaretByWord({required int direction, bool extend = false}) {
    _selection.moveGlyphCaretByWord(direction: direction, extend: extend);
    ensureCaretVisible(notify: false);
  }

  void moveGlyphCaretByParagraph({required int direction, bool extend = false}) {
    _selection.moveGlyphCaretByParagraph(direction: direction, extend: extend);
    ensureCaretVisible(notify: false);
  }

  /// Single dispatcher for keyboard input — every path must call this.
  Future<void> handleEditorInput(EditorInputEvent event) async {
    if (hasSelectedDiagram && _isNavigationInput(event)) {
      final entered = await enterSelectedShapeTextEdit();
      if (!entered) return;
      // Fall through so the navigation key still moves within the table/shape.
    }
    final extend = event.extendsSelection;
    switch (event.kind) {
      case EditorInputKind.character:
        await insertGlyphCharacter(event.character!);
      case EditorInputKind.newline:
        await insertGlyphParagraphBreak();
      case EditorInputKind.lineBreak:
        await insertGlyphLineBreak();
      case EditorInputKind.pageBreak:
        insertPageBreak();
      case EditorInputKind.tab:
        // Word only changes the list level from the start of the list
        // paragraph; Tab anywhere else in the text inserts a tab character.
        // Shift+Tab promotes from anywhere in the item.
        // Inside SmartArt / multi-cell shapes, Tab jumps to the next node.
        if (event.shift) {
          if (isInList) {
            demoteListLevel();
          } else if (!_selection.tryMoveGlyphCaretToAdjacentBlock(direction: -1)) {
            decreaseIndent();
          }
        } else if (isInList && caretOffset == 0) {
          promoteListLevel();
        } else if (_selection.tryMoveGlyphCaretToAdjacentBlock(direction: 1)) {
          ensureCaretVisible(notify: false);
        } else {
          await insertGlyphCharacter('\t');
        }
      case EditorInputKind.backspace:
        await deleteGlyphBackward();
      case EditorInputKind.delete:
        await deleteGlyphForward();
      case EditorInputKind.deleteWordBackward:
        await deleteGlyphWord(forward: false);
      case EditorInputKind.deleteWordForward:
        await deleteGlyphWord(forward: true);
      case EditorInputKind.arrowLeft:
        moveGlyphCaretByArrow(LogicalKeyboardKey.arrowLeft, extend: extend);
      case EditorInputKind.arrowRight:
        moveGlyphCaretByArrow(LogicalKeyboardKey.arrowRight, extend: extend);
      case EditorInputKind.arrowUp:
        moveGlyphCaretByArrow(LogicalKeyboardKey.arrowUp, extend: extend);
      case EditorInputKind.arrowDown:
        moveGlyphCaretByArrow(LogicalKeyboardKey.arrowDown, extend: extend);
      case EditorInputKind.wordLeft:
        moveGlyphCaretByWord(direction: -1, extend: extend);
      case EditorInputKind.wordRight:
        moveGlyphCaretByWord(direction: 1, extend: extend);
      case EditorInputKind.paragraphUp:
        moveGlyphCaretByParagraph(direction: -1, extend: extend);
      case EditorInputKind.paragraphDown:
        moveGlyphCaretByParagraph(direction: 1, extend: extend);
      case EditorInputKind.lineStart:
        moveGlyphCaretToLineEdge(toEnd: false, extend: extend);
      case EditorInputKind.lineEnd:
        moveGlyphCaretToLineEdge(toEnd: true, extend: extend);
      case EditorInputKind.documentStart:
        moveGlyphCaretToDocumentEdge(toEnd: false, extend: extend);
      case EditorInputKind.documentEnd:
        moveGlyphCaretToDocumentEdge(toEnd: true, extend: extend);
      case EditorInputKind.pageUp:
        moveGlyphCaretByPage(direction: -1, extend: extend);
      case EditorInputKind.pageDown:
        moveGlyphCaretByPage(direction: 1, extend: extend);
    }
  }

  /// Scroll the canvas so the caret stays on-screen after it moves.
  ///
  /// When [notify] is false the request is queued for the end of this frame
  /// (the caller already notified, or will notify next).
  void ensureCaretVisible({bool notify = true}) {
    final page = _selection.caretPage;
    final pageChanged = page != _view.currentPage;
    if (pageChanged) {
      _selectPage(page);
      requestEditorFocus();
    }
    final geom = _selection.caretGeometry;
    if (geom != null) {
      _view.requestScrollToCaret(page: page, y: geom.y, height: geom.height);
    } else {
      _view.requestScrollToPage(page);
      return;
    }
    if (notify) notifyListeners();
  }

  void beginGlyphSelection(int p, double x, double y) {
    // Word exits header/footer edit when the user clicks the body.
    if (_editZone == DocumentEditZone.header && y > marginTop) {
      closeHeaderFooterEdit();
    } else if (_editZone == DocumentEditZone.footer &&
        y < pageHeight - marginBottom) {
      closeHeaderFooterEdit();
    }
    _selection.beginGlyphSelection(p, x, y);
  }
  void updateGlyphSelection(int p, double x, double y) => _selection.updateGlyphSelection(p, x, y);
  void endGlyphSelection(int p, double x, double y) => _selection.endGlyphSelection(p, x, y);
  bool isPointInGlyphSelection(int p, Offset pt) => _selection.isPointInGlyphSelection(p, pt);
  void beginGlyphDrag(int p) => _selection.beginGlyphDrag(p);
  void updateGlyphDragDropCaret(int p, double x, double y) =>
      _selection.updateGlyphDragDropCaret(p, x, y);
  void completeGlyphDrag(int p, double x, double y) => _completeGlyphDrag(p, x, y);
  void cancelGlyphDrag() => _selection.cancelGlyphDrag();
  bool get isGlyphDragActive => _selection.isGlyphDragActive;

  bool get hasSelectedImage => _selectedImageId != null;
  String? get selectedImageId => _selectedImageId;
  int? get selectedImagePage => _selectedImagePage;
  Rect? get selectedImageRect => _previewImageRect ?? _selectedImageRect;
  bool get hasSelectedDiagram => _selectedDiagramId != null;
  String? get selectedDiagramId => _selectedDiagramId;
  int? get selectedDiagramPage => _selectedDiagramPage;
  Rect? get selectedDiagramRect => _previewDiagramRect ?? _selectedDiagramRect;
  bool get isImageResizing => _activeImageHandle != null;

  /// True while a selected image/shape is being moved or resized.
  ///
  /// On iPhone/iPad the document [ListView] otherwise wins the vertical-drag
  /// arena and scrolls under the finger (or cancels the object move).
  bool get locksDocumentScroll =>
      _moveStartPoint != null || _resizeStartRect != null;

  /// Original object bounds while a move preview is active (content drag).
  Rect? get contentDragSourceRect {
    if (_previewDiagramRect != null) return _selectedDiagramRect;
    if (_previewImageRect != null) return _selectedImageRect;
    return null;
  }

  /// Pixel offset from [contentDragSourceRect] to the live preview position.
  Offset get contentDragOffset {
    final origin = contentDragSourceRect;
    final preview = _previewDiagramRect ?? _previewImageRect;
    if (origin == null || preview == null) return Offset.zero;
    return preview.topLeft - origin.topLeft;
  }

  void selectImage(int pageIndex, ImageBounds bounds) {
    clearDiagramSelection();
    _selectedImageId = bounds.imageId;
    _selectedImagePage = pageIndex;
    _selectedImageRect = bounds.rect;
    _previewImageRect = null;
    _selectedImageRotation = 0;
    notifyListeners();
  }

  void clearImageSelection() {
    if (_selectedImageId == null &&
        _selectedImageRect == null &&
        _previewImageRect == null) {
      return;
    }
    _selectedImageId = null;
    _selectedImagePage = null;
    _selectedImageRect = null;
    _previewImageRect = null;
    _selectedImageRotation = 0;
    _activeImageHandle = null;
    _resizeStartRect = null;
    notifyListeners();
  }

  bool trySelectImageAt(int pageIndex, Offset point, DisplayListSnapshot snapshot) {
    final hit = hitTestImage(snapshot, point);
    if (hit == null) {
      clearImageSelection();
      return false;
    }
    selectImage(pageIndex, hit);
    return true;
  }

  void selectDiagram(int pageIndex, ShapeBounds bounds) {
    clearImageSelection();
    _selection.collapseToCaret();
    // Word hides the text caret while object handles are shown. Leaving a
    // caret inside the shape made Backspace delete the whole object (or no-op)
    // instead of editing characters.
    _selection.clearCaretVisual();
    _shapeTextEditActive = false;
    _shapeTextEditShapeId = null;
    _selectedDiagramId = bounds.shapeId;
    _selectedDiagramPage = pageIndex;
    _selectedDiagramRect = bounds.rect;
    _previewDiagramRect = null;
    notifyListeners();
  }

  void clearDiagramSelection() {
    if (_selectedDiagramId == null &&
        _selectedDiagramRect == null &&
        _previewDiagramRect == null) {
      return;
    }
    _selectedDiagramId = null;
    _selectedDiagramPage = null;
    _selectedDiagramRect = null;
    _previewDiagramRect = null;
    notifyListeners();
  }

  /// Selects a chart / SmartArt / shape / table object.
  ///
  /// Border clicks keep object selection. A second interior click (or a click
  /// onto another SmartArt node while already typing in the diagram) places
  /// the caret in the text under the pointer.
  bool trySelectDiagramAt(int pageIndex, Offset point, DisplayListSnapshot snapshot) {
    final hit = hitTestShape(snapshot, point);
    if (hit == null) {
      _shapeTextEditActive = false;
      _shapeTextEditShapeId = null;
      clearDiagramSelection();
      return false;
    }

    final nearBorder = hit.containsNearBorder(point);
    final alreadySelected = _selectedDiagramId == hit.shapeId;

    if (!nearBorder && alreadySelected) {
      unawaited(enterSelectedShapeTextEdit(at: point));
      return true;
    }

    // Already typing in this SmartArt/shape: move caret to the clicked node
    // instead of re-selecting the whole object.
    if (!nearBorder &&
        _shapeTextEditActive &&
        _shapeTextEditShapeId == hit.shapeId) {
      _selection.beginGlyphSelection(pageIndex, point.dx, point.dy);
      requestEditorFocus();
      notifyListeners();
      return true;
    }

    selectDiagram(pageIndex, hit);
    return true;
  }

  /// Places the caret inside the selected shape so typing edits its body text.
  ///
  /// Returns false when the selection is missing or the shape cannot host text
  /// (charts, lines). Tables succeed via [ensureShapeTextAsync] (no-op) and
  /// hit-testing into a cell. When [at] is set (click into a SmartArt node),
  /// that point wins over the EnsureShapeText seed run.
  Future<bool> enterSelectedShapeTextEdit({Offset? at}) async {
    final id = _selectedDiagramId;
    final page = _selectedDiagramPage;
    final rect = _selectedDiagramRect;
    if (id == null || page == null || rect == null || _host.engine == null) {
      return false;
    }

    final ensured = await _host.performNativeEdit(
      () => _host.engine!.ensureShapeTextAsync(id),
      full: true,
    );
    if (!ensured) {
      return false;
    }
    await _host.ensureLayoutReady();

    _shapeTextEditActive = true;
    _shapeTextEditShapeId = id;
    clearDiagramSelection();

    if (at != null) {
      final direct = _host.engine!.hitTestPage(page, at.dx, at.dy);
      if (direct != null) {
        _selection.beginGlyphSelection(page, at.dx, at.dy);
        _selection.syncCaretGeometry();
        if (_selection.caretRunId == direct.runId) {
          requestEditorFocusAfterOverlay();
          notifyListeners();
          return true;
        }
      }
    }

    final seed = _host.engine!.fetchLastSplitCaret();
    for (final probe in _shapeTextProbes(rect)) {
      final hit = _host.engine!.hitTestPage(page, probe.dx, probe.dy);
      if (hit == null) continue;
      if (seed != null && hit.runId != seed.runId) continue;
      _selection.beginGlyphSelection(page, probe.dx, probe.dy);
      _selection.syncCaretGeometry();
      if (_selection.caretRunId == hit.runId) {
        requestEditorFocusAfterOverlay();
        notifyListeners();
        return true;
      }
    }

    if (seed != null) {
      final existing = _host.engine!.fetchTextRange(
            seed.runId,
            0,
            seed.runId,
            1 << 16,
          ) ??
          '';
      _selection.setCaret(seed.runId, existing.length, page: page);
      _selection.syncCaretGeometry();
      requestEditorFocusAfterOverlay();
      notifyListeners();
      return true;
    }

    // No seed (e.g. mock): accept any text hit inside the shape frame.
    for (final probe in _shapeTextProbes(rect)) {
      final hit = _host.engine!.hitTestPage(page, probe.dx, probe.dy);
      if (hit == null) continue;
      _selection.beginGlyphSelection(page, probe.dx, probe.dy);
      _selection.syncCaretGeometry();
      if (_selection.defaultRunId() != null) {
        requestEditorFocus();
        notifyListeners();
        return true;
      }
    }
    notifyListeners();
    return false;
  }

  List<Offset> _shapeTextProbes(Rect rect) {
    final probes = <Offset>[
      Offset(rect.left + 10.0, rect.top + 14.0),
      Offset(rect.left + 10.0, rect.top + rect.height * 0.45),
      Offset(rect.left + rect.width * 0.25, rect.top + rect.height * 0.35),
      Offset(rect.left + rect.width * 0.5, rect.top + rect.height * 0.55),
      Offset(rect.left + rect.width * 0.22, rect.top + rect.height * 0.55),
      Offset(rect.left + 8.0, rect.top + 8.0),
      Offset(rect.center.dx, rect.center.dy),
    ];
    for (var i = 1; i <= 4; i++) {
      for (var j = 1; j <= 4; j++) {
        probes.add(Offset(
          rect.left + rect.width * i / 5,
          rect.top + rect.height * j / 5,
        ));
      }
    }
    return probes;
  }

  /// Ensures the caret is inside a table cell before Layout → Table actions.
  ///
  /// Object-selected tables clear the text caret; Sort / Sum / Nested need a
  /// cell run id.
  Future<bool> _prepareTableCellEdit() async {
    if (!_host.isConnected) return false;
    if (hasSelectedDiagram) {
      final entered = await enterSelectedShapeTextEdit();
      if (!entered) {
        _session.setStatusText('Click inside a table cell first');
        notifyListeners();
        return false;
      }
    }
    requestEditorFocus();
    return true;
  }

  bool get hasSelectedObject => hasSelectedDiagram || hasSelectedImage;

  Future<bool> deleteSelectedObject() async {
    if (!_host.isConnected || _host.engine == null) return false;
    final blockId = _selectedDiagramId ?? _selectedImageId;
    if (blockId == null) return false;
    final ok = await _host.performNativeEdit(
      () => _host.engine!.deleteBlockAsync(blockId),
      full: true,
    );
    if (ok) {
      clearDiagramSelection();
      clearImageSelection();
      _session.markDocumentDirty();
      _session.setStatusText('Object deleted');
      notifyListeners();
    }
    return ok;
  }

  ImageResizeHandle? imageHandleAt(Offset point) {
    final rect = selectedImageRect;
    if (rect == null) return null;
    return hitTestImageHandle(rect, point);
  }

  void beginImageResize(ImageResizeHandle handle) {
    final rect = _selectedImageRect;
    if (rect == null) return;
    _activeImageHandle = handle;
    _resizeStartRect = rect;
    _previewImageRect = rect;
    notifyListeners();
  }

  void updateImageResize(Offset current, {required bool lockAspectRatio}) {
    final start = _resizeStartRect;
    final handle = _activeImageHandle;
    if (start == null || handle == null) return;
    _previewImageRect = resizeImageWithHandle(
      start: start,
      handle: handle,
      current: current,
      origin: start.topLeft,
      lockAspectRatio: lockAspectRatio,
    );
    notifyListeners();
  }

  Future<void> commitImageResize() async {
    final id = _selectedImageId;
    final rect = _previewImageRect ?? _selectedImageRect;
    _activeImageHandle = null;
    _resizeStartRect = null;
    if (id == null || rect == null || !_host.isConnected) {
      _previewImageRect = null;
      notifyListeners();
      return;
    }
    await _session.applyEngineStyle(
      () => _host.engine!.setImageSizeAsync(id, rect.width, rect.height),
      'Image resized',
      full: true,
    );
    _selectedImageRect = rect;
    _previewImageRect = null;
    notifyListeners();
  }

  void cancelImageResize() {
    _activeImageHandle = null;
    _resizeStartRect = null;
    _previewImageRect = null;
    notifyListeners();
  }

  Future<void> replaceSelectedImage() async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    _session.setStatusText('Choose picture…');
    notifyListeners();
    try {
      final picked = await pickDocumentFile(
        dialogTitle: 'Replace picture',
        allowedExtensions: const ['png', 'jpg', 'jpeg', 'svg'],
      );
      if (picked == null) {
        _session.setStatusText('Replace cancelled');
        return;
      }
      final mime = _mimeForPicture(picked.name.split('.').last, picked.name);
      await replaceSelectedImageBytes(picked.bytes, mime);
    } catch (e) {
      _session.setStatusText('Replace failed: $e');
    }
  }

  Future<void> replaceSelectedImageBytes(Uint8List bytes, String mimeType) async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.replaceImageBytesAsync(id, bytes, mimeType),
      'Picture replaced',
      full: true,
    );
    notifyListeners();
  }

  bool isPointOnSelectedImage(Offset point) {
    final rect = selectedImageRect;
    return rect != null && rect.contains(point);
  }

  void beginImageMove(Offset point) {
    final rect = _selectedImageRect;
    if (rect == null) return;
    _moveStartPoint = point;
    _moveStartRect = rect;
    _previewImageRect = rect;
    notifyListeners();
  }

  void updateImageMove(Offset current) {
    final start = _moveStartPoint;
    final origin = _moveStartRect;
    if (start == null || origin == null) return;
    _previewImageRect = origin.shift(current - start);
    notifyListeners();
  }

  Future<void> commitImageMove() async {
    final id = _selectedImageId;
    final rect = _previewImageRect ?? _selectedImageRect;
    _moveStartPoint = null;
    _moveStartRect = null;
    if (id == null || rect == null || !_host.isConnected) {
      _previewImageRect = null;
      notifyListeners();
      return;
    }
    final ok = await _session.applyEngineStyle(
      () => _host.engine!.setImageAnchorAsync(
        id,
        rect.left - marginLeft,
        rect.top - marginTop,
      ),
      'Image moved',
      full: true,
    );
    if (ok) {
      _selectedImageRect = rect;
    }
    _previewImageRect = null;
    notifyListeners();
  }

  void cancelImageMove() {
    _moveStartPoint = null;
    _moveStartRect = null;
    _previewImageRect = null;
    notifyListeners();
  }

  bool isPointOnSelectedDiagram(Offset point) {
    final rect = selectedDiagramRect;
    return rect != null && rect.contains(point);
  }

  void beginShapeMove(Offset point) {
    final rect = _selectedDiagramRect;
    if (rect == null) return;
    _moveStartPoint = point;
    _moveStartRect = rect;
    _previewDiagramRect = rect;
    notifyListeners();
  }

  void updateShapeMove(Offset current) {
    final start = _moveStartPoint;
    final origin = _moveStartRect;
    if (start == null || origin == null) return;
    _previewDiagramRect = origin.shift(current - start);
    notifyListeners();
  }

  Future<void> commitShapeMove() async {
    final id = _selectedDiagramId;
    final rect = _previewDiagramRect ?? _selectedDiagramRect;
    _moveStartPoint = null;
    _moveStartRect = null;
    if (id == null || rect == null || !_host.isConnected) {
      _previewDiagramRect = null;
      notifyListeners();
      return;
    }
    final ok = await _session.applyEngineStyle(
      () => _host.engine!.setShapeAnchorAsync(
        id,
        rect.left - marginLeft,
        rect.top - marginTop,
      ),
      'Object moved',
      full: true,
    );
    _previewDiagramRect = null;
    if (ok) {
      await _host.ensureLayoutReady();
      selectDiagramById(id);
    }
    notifyListeners();
  }

  void cancelShapeMove() {
    _moveStartPoint = null;
    _moveStartRect = null;
    _previewDiagramRect = null;
    notifyListeners();
  }

  Future<void> setSelectedImageWrap(int wrap) async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    final label = switch (wrap) {
      0 => 'Inline with text',
      1 => 'Square wrap',
      3 => 'Behind text',
      _ => 'Wrap updated',
    };
    await _session.applyEngineStyle(
      () => _host.engine!.setImageWrapAsync(id, wrap),
      label,
      full: true,
    );
    notifyListeners();
  }

  Future<void> rotateSelectedImage({bool clockwise = true}) async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    _selectedImageRotation =
        (_selectedImageRotation + (clockwise ? 90 : -90)) % 360;
    await _session.applyEngineStyle(
      () => _host.engine!.setImageTransformAsync(
        id,
        rotationDeg: _selectedImageRotation,
      ),
      'Picture rotated',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertSelectedImageCaption() async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertImageCaptionAsync(id),
      'Caption inserted',
      full: true,
    );
    notifyListeners();
  }

  /// Current alt text for the selected image (empty when unset) — F21.S3.
  String get selectedImageAltText {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected || _host.engine == null) return '';
    return _host.engine!.fetchImageAltText(id) ?? '';
  }

  Uint8List? fetchImageAssetBytes(String assetId) =>
      _host.engine?.fetchImageAssetBytes(assetId);

  Future<void> setSelectedImageAltText(String? altText) async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.setImageAltTextAsync(id, altText),
      'Alt text updated',
      full: true,
    );
    notifyListeners();
  }

  /// Generate alt text for the selected image via AI.
  Future<void> generateAutoAltText(BuildContext context) async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    _session.setStatusText('Generating alt text…');
    notifyListeners();
    try {
      final alt = await _aiClient.rewrite(
        AiDocumentContext(
          selectionText:
              'Write one concise accessibility alt text sentence for a document image. '
              'Return only the alt text.',
          totalTokenEstimate: 120,
        ),
        AiRewriteTone.neutral,
      );
      if (!context.mounted) return;
      await setSelectedImageAltText(alt.trim());
      _session.setStatusText('Alt text generated');
    } catch (e) {
      _session.setStatusText('Alt text generation failed: $e');
    }
    notifyListeners();
  }

  Future<void> compressSelectedImage({int quality = 75}) async {
    final id = _selectedImageId;
    if (id == null || !_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.compressImageAsync(id, quality),
      'Picture compressed',
      full: true,
    );
    notifyListeners();
  }

  void selectGlyphWordAt(int p, double x, double y) => _selection.selectGlyphWordAt(p, x, y);
  Future<void> selectAll() => _selection.selectAll();

  // ── Formatting ────────────────────────────────────────────────────────────
  bool get bold => _formatting.bold;
  bool get italic => _formatting.italic;
  bool get underline => _formatting.underline;
  String get fontFamily => _formatting.fontFamily;
  double get fontSize => _formatting.fontSize;
  TextAlign get alignment => _formatting.alignment;
  bool get strikethrough => _formatting.strikethrough;
  bool get subscript => _formatting.subscript;
  bool get superscript => _formatting.superscript;
  bool get allCaps => _formatting.allCaps;
  bool get smallCaps => _formatting.smallCaps;
  bool get hidden => _formatting.hidden;
  bool get ligatures => _formatting.ligatures;
  Color get fontColor => _formatting.fontColor;
  Color? get highlightColor => _formatting.highlightColor;
  String get activeParagraphStyle => _formatting.activeParagraphStyle;
  String get documentThemeName => _documentThemeName;
  double get indentLeft => _formatting.indentLeft;
  double get indentFirstLine => _formatting.indentFirstLine;
  LineSpacingMode get lineSpacing => _formatting.lineSpacing;
  double get exactLineSpacingPt => _formatting.exactLineSpacingPt;
  double get spaceBefore => _formatting.spaceBefore;
  double get spaceAfter => _formatting.spaceAfter;
  List<Map<String, dynamic>> get tabStops => _formatting.tabStops;

  bool get formatPainterArmed => _formatting.formatPainterArmed;

  void toggleFormatPainter() {
    if (_formatting.formatPainterArmed) {
      _formatting.cancelFormatPainter();
      _session.setStatusText('Format Painter cancelled');
    } else {
      final ok = _formatting.pickupFormatPainter();
      _session.setStatusText(
        ok
            ? 'Format Painter: select text to paint'
            : 'Format Painter: place the caret in formatted text',
      );
    }
    notifyListeners();
  }

  Future<void> _onSelectionChangedFormatPainter() async {
    final applied = await _formatting.applyFormatPainterIfArmed();
    if (!applied) return;
    _session.setStatusText('Format painted');
    notifyListeners();
  }

  /// Ctrl+Shift+C — pick up the format under the caret, without the ribbon's
  /// toggle semantics: pressing it twice re-copies rather than cancelling.
  void copyFormatting() {
    final ok = _formatting.pickupFormatPainter();
    _session.setStatusText(
      ok ? 'Formatting copied' : 'Place the caret in formatted text first',
    );
    notifyListeners();
  }

  /// Ctrl+Shift+V — lay the copied format onto the selection or caret.
  Future<void> pasteFormatting() async {
    final applied = await _formatting.applyFormatPainter();
    _session.setStatusText(
      applied ? 'Formatting pasted' : 'Copy formatting first (Ctrl+Shift+C)',
    );
    notifyListeners();
  }

  void toggleBold() => _formatting.toggleBold();
  void toggleItalic() => _formatting.toggleItalic();
  void toggleUnderline() => _formatting.toggleUnderline();
  void setFontFamily(String f) => _formatting.setFontFamily(f);
  void setFontSize(double s) => _formatting.setFontSize(s);
  void increaseFontSize() => _formatting.increaseFontSize();
  void decreaseFontSize() => _formatting.decreaseFontSize();
  void setFontColor(Color c, {String? themeSlot, int? themeVariant}) =>
      _formatting.setFontColor(c, themeSlot: themeSlot, themeVariant: themeVariant);
  void clearFontColor() => _formatting.clearFontColor();
  void setHighlight(Color c) => _formatting.setHighlight(c);
  void clearHighlight() => _formatting.clearHighlight();
  void setAlignment(TextAlign a) => _formatting.setAlignment(a);
  void toggleStrikethrough() => _formatting.toggleStrikethrough();
  void toggleSubscript() => _formatting.toggleSubscript();
  void toggleSuperscript() => _formatting.toggleSuperscript();
  void toggleAllCaps() => _formatting.toggleAllCaps();
  void toggleSmallCaps() => _formatting.toggleSmallCaps();
  void toggleHidden() => _formatting.toggleHidden();
  void toggleLigatures() => _formatting.toggleLigatures();
  void clearFormatting() => unawaited(_formatting.clearFormatting());
  void increaseIndent() => _formatting.increaseIndent();
  void decreaseIndent() => _formatting.decreaseIndent();
  bool get isInList => _formatting.isInList;
  int get listLevel => _formatting.listLevel;
  void promoteListLevel() => _formatting.promoteListLevel();
  void demoteListLevel() => _formatting.demoteListLevel();

  /// Ctrl+Q — remove direct paragraph formatting.
  void clearParagraphFormatting() => _formatting.clearParagraphFormatting();

  /// Ctrl+T / Ctrl+Shift+T — grow or shrink the hanging indent.
  void adjustHangingIndent({required bool increase}) =>
      _formatting.adjustHangingIndent(increase: increase);

  /// Ctrl+0 — toggle 12 pt of space above the paragraph.
  void toggleSpaceBefore() => _formatting.toggleSpaceBefore();

  /// Alt+Shift+Up / Down — move the caret's paragraph, reporting when it was
  /// already at the edge so the status line can say so.
  Future<void> moveParagraph({required int direction}) async {
    final moved = await _formatting.moveParagraph(direction: direction);
    _session.setStatusText(
      moved
          ? direction < 0
              ? 'Paragraph moved up'
              : 'Paragraph moved down'
          : 'Paragraph is already at the edge',
    );
    notifyListeners();
  }
  void applySpacing({
    required LineSpacingMode lineSpacing,
    required double exactPoints,
    required double spaceBefore,
    required double spaceAfter,
    required bool keepTogether,
    required bool keepWithNext,
    required bool widowOrphanControl,
  }) =>
      _formatting.applySpacing(
        lineSpacing: lineSpacing,
        exactPoints: exactPoints,
        spaceBefore: spaceBefore,
        spaceAfter: spaceAfter,
        keepTogether: keepTogether,
        keepWithNext: keepWithNext,
        widowOrphanControl: widowOrphanControl,
      );

  /// Ctrl+1 / Ctrl+2 / Ctrl+5 — change line spacing, leaving the paragraph's
  /// other spacing settings alone.
  void applyLineSpacing(LineSpacingMode mode) => applySpacing(
        lineSpacing: mode,
        exactPoints: _formatting.exactLineSpacingPt,
        spaceBefore: _formatting.spaceBefore,
        spaceAfter: _formatting.spaceAfter,
        keepTogether: _formatting.keepTogether,
        keepWithNext: _formatting.keepWithNext,
        widowOrphanControl: _formatting.widowOrphanControl,
      );

  bool get keepTogether => _formatting.keepTogether;
  bool get keepWithNext => _formatting.keepWithNext;
  bool get widowOrphanControl => _formatting.widowOrphanControl;
  Color? get paraShading => _formatting.paraShading;
  double get borderWidth => _formatting.borderWidth;

  void applyBordersAndShading({
    Color? shading,
    double borderWidth = 0,
    Color borderColor = Colors.black,
    bool clearShading = false,
    bool clearBorders = false,
  }) =>
      _formatting.applyBordersAndShading(
        shading: shading,
        borderWidth: borderWidth,
        borderColor: borderColor,
        clearShading: clearShading,
        clearBorders: clearBorders,
      );

  Future<void> showParagraphSpacingDialog(BuildContext context) async {
    final values = await ParagraphSpacingDialog.show(
      context,
      initial: ParagraphSpacingValues(
        lineSpacing: _formatting.lineSpacing,
        exactPoints: _formatting.exactLineSpacingPt,
        spaceBefore: _formatting.spaceBefore,
        spaceAfter: _formatting.spaceAfter,
        keepTogether: _formatting.keepTogether,
        keepWithNext: _formatting.keepWithNext,
        widowOrphanControl: _formatting.widowOrphanControl,
      ),
    );
    if (values == null) return;
    applySpacing(
      lineSpacing: values.lineSpacing,
      exactPoints: values.exactPoints,
      spaceBefore: values.spaceBefore,
      spaceAfter: values.spaceAfter,
      keepTogether: values.keepTogether,
      keepWithNext: values.keepWithNext,
      widowOrphanControl: values.widowOrphanControl,
    );
  }

  void increaseSpaceBefore() {
    applySpacing(
      lineSpacing: _formatting.lineSpacing,
      exactPoints: _formatting.exactLineSpacingPt,
      spaceBefore: (_formatting.spaceBefore + 6).clamp(0, 240),
      spaceAfter: _formatting.spaceAfter,
      keepTogether: _formatting.keepTogether,
      keepWithNext: _formatting.keepWithNext,
      widowOrphanControl: _formatting.widowOrphanControl,
    );
    _session.setStatusText('Space before: ${_formatting.spaceBefore.round()} pt');
    notifyListeners();
  }

  void increaseSpaceAfter() {
    applySpacing(
      lineSpacing: _formatting.lineSpacing,
      exactPoints: _formatting.exactLineSpacingPt,
      spaceBefore: _formatting.spaceBefore,
      spaceAfter: (_formatting.spaceAfter + 6).clamp(0, 240),
      keepTogether: _formatting.keepTogether,
      keepWithNext: _formatting.keepWithNext,
      widowOrphanControl: _formatting.widowOrphanControl,
    );
    _session.setStatusText('Space after: ${_formatting.spaceAfter.round()} pt');
    notifyListeners();
  }

  /// Home → Change Case on the current selection.
  Future<void> applyChangeCase(ChangeCaseKind kind) async {
    if (!_host.isConnected || _host.engine == null) return;
    if (!_selection.hasGlyphSelection) {
      _session.setStatusText('Select text to change case');
      notifyListeners();
      return;
    }
    final range = _selection.selection;
    if (range == null) return;
    final (start, end) = range.normalized();
    if (start.runId != end.runId) {
      _session.setStatusText('Change case supports a single-run selection');
      notifyListeners();
      return;
    }
    final original = selectedText;
    if (original.isEmpty) return;
    final transformed = transformChangeCase(original, kind);
    final lo = start.offset < end.offset ? start.offset : end.offset;
    final hi = start.offset < end.offset ? end.offset : start.offset;
    final ok = await applyAiTextSuggestion(
      runId: start.runId,
      start: lo,
      end: hi,
      text: transformed,
    );
    _session.setStatusText(ok ? 'Case changed' : 'Change case failed');
    notifyListeners();
  }

  /// Shift+F3 — Word's case cycle. The next case is read off the selection
  /// rather than a press counter, so the cycle survives clicking elsewhere and
  /// back: lower case → Title Case → UPPER CASE → lower case.
  Future<void> cycleChangeCase() async {
    final text = selectedText;
    final isAllUpper = text == text.toUpperCase() && text != text.toLowerCase();
    final kind = isAllUpper
        ? ChangeCaseKind.lower
        : text == text.toLowerCase()
            ? ChangeCaseKind.capitalizeEachWord
            : ChangeCaseKind.upper;
    final range = _selection.selection;
    await applyChangeCase(kind);
    // Word leaves the text selected so the chord can be pressed again. The
    // range still describes it as long as the case map kept the length — ß → SS
    // is the exception, and there the caret is left where the edit ended.
    if (range != null && transformChangeCase(text, kind).length == text.length) {
      _selection.selectDocRange(range);
    }
  }

  /// Home → Sort paragraphs A→Z or Z→A (selection or whole document).
  Future<void> sortParagraphs({required bool ascending}) async {
    if (!_host.isConnected || _host.engine == null) return;
    final runId = _selection.defaultRunId() ??
        (_host.engine is MockDocumentEngine
            ? (_host.engine as MockDocumentEngine).defaultRunId
            : null);
    if (runId == null) {
      _session.setStatusText('Sort failed: no caret run');
      notifyListeners();
      return;
    }

    String source;
    int replaceStart;
    int replaceEnd;
    if (_selection.hasGlyphSelection) {
      final range = _selection.selection;
      if (range == null) return;
      final (start, end) = range.normalized();
      if (start.runId != end.runId) {
        _session.setStatusText('Sort supports a single-run selection');
        notifyListeners();
        return;
      }
      source = selectedText;
      replaceStart = start.offset < end.offset ? start.offset : end.offset;
      replaceEnd = start.offset < end.offset ? end.offset : start.offset;
    } else {
      source = documentText;
      replaceStart = 0;
      replaceEnd = source.length;
    }

    final paragraphs = source.split('\n');
    paragraphs.sort((a, b) {
      final cmp = a.trim().toLowerCase().compareTo(b.trim().toLowerCase());
      return ascending ? cmp : -cmp;
    });
    final sorted = paragraphs.join('\n');
    final ok = await applyAiTextSuggestion(
      runId: runId,
      start: replaceStart,
      end: replaceEnd,
      text: sorted,
    );
    _session.setStatusText(
      ok
          ? ascending
              ? 'Paragraphs sorted A→Z'
              : 'Paragraphs sorted Z→A'
          : 'Sort failed',
    );
    notifyListeners();
  }

  Future<void> showParagraphBordersDialog(BuildContext context) async {
    final values = await ParagraphBordersDialog.show(
      context,
      initial: ParagraphBordersValues(
        shading: _formatting.paraShading,
        borderWidth: _formatting.borderWidth,
      ),
    );
    if (values == null) return;
    applyBordersAndShading(
      shading: values.shading,
      borderWidth: values.borderWidth,
      borderColor: values.borderColor,
      clearShading: values.clearShading,
      clearBorders: values.clearBorders,
    );
  }

  void applyTabStops(List<Map<String, dynamic>> stops) =>
      _formatting.applyTabStops(stops);

  Future<void> showTabStopsDialog(BuildContext context) async {
    final initial = _formatting.tabStops
        .map(TabStopValue.fromJson)
        .whereType<TabStopValue>()
        .toList();
    final result = await TabStopsDialog.show(context, initial: initial);
    if (result == null) return;
    applyTabStops(result.map((s) => s.toJson()).toList());
  }

  // ── Clipboard ─────────────────────────────────────────────────────────────
  String get selectedText => _selection.selectedText();

  bool get canCutOrCopy => selectedText.isNotEmpty;
  bool get canDelete => canCutOrCopy || hasSelectedObject;

  // ── Read aloud (F21.S5) ───────────────────────────────────────────────────
  bool get isReadingAloud => _readingAloud || _tts.isSpeaking;

  /// Speaks [text] in paragraph-bounded chunks so platform TTS limits are not hit.
  Future<void> _speakChunked(String text) async {
    const maxChunkChars = 3000;
    final chunks = <String>[];
    for (final block in text.split(RegExp(r'\n{2,}'))) {
      final trimmed = block.trim();
      if (trimmed.isEmpty) continue;
      if (trimmed.length <= maxChunkChars) {
        chunks.add(trimmed);
        continue;
      }
      var start = 0;
      while (start < trimmed.length) {
        final end = (start + maxChunkChars).clamp(0, trimmed.length);
        chunks.add(trimmed.substring(start, end));
        start = end;
      }
    }
    for (final chunk in chunks) {
      if (!_readingAloud) break;
      await _tts.speak(chunk);
    }
  }

  /// Speaks the current selection, or the whole document when nothing is
  /// selected — Word's Read Aloud falls back to the document rather than
  /// requiring a selection first.
  Future<void> readAloudSelection() async {
    final selection = selectedText.trim();
    final text = selection.isNotEmpty ? selection : documentText.trim();
    if (text.isEmpty) {
      _session.setStatusText('Nothing to read aloud');
      notifyListeners();
      return;
    }
    if (_readingAloud) {
      await stopReadAloud();
    }
    _readingAloud = true;
    _session.setStatusText(
      selection.isNotEmpty ? 'Reading selection…' : 'Reading document…',
    );
    notifyListeners();
    try {
      await _speakChunked(text);
      if (_readingAloud) {
        _session.setStatusText('Finished reading aloud');
      }
    } catch (_) {
      _session.setStatusText('Read aloud unavailable');
    } finally {
      _readingAloud = false;
      notifyListeners();
    }
  }

  Future<void> stopReadAloud() async {
    final wasReading = _readingAloud || _tts.isSpeaking;
    _readingAloud = false;
    await _tts.stop();
    if (wasReading) {
      _session.setStatusText('Read aloud stopped');
    }
    notifyListeners();
  }

  Future<void> toggleReadAloud() async {
    if (isReadingAloud) {
      await stopReadAloud();
    } else {
      await readAloudSelection();
    }
  }

  Uint8List? _lastClipboardDocx;

  Future<void> copySelection() async {
    final text = selectedText;
    if (text.isEmpty) return;
    Uint8List? docxBytes;
    if (hasGlyphSelection && _host.engine != null) {
      final range = selection;
      if (range != null) {
        final (start, end) = range.normalized();
        docxBytes = await _host.engine!.exportSelectionDocxAsync(
          startRunId: start.runId,
          startOffset: start.offset,
          endRunId: end.runId,
          endOffset: end.offset,
        );
      }
    }
    _lastClipboardDocx = docxBytes;
    await Clipboard.setData(ClipboardData(text: text));
  }

  Future<void> cutSelection() async {
    if (hasSelectedObject) {
      await deleteSelectedObject();
      return;
    }
    if (!canCutOrCopy) return;
    final text = selectedText;
    await Clipboard.setData(ClipboardData(text: text));
    await _selection.deleteGlyphSelection();
    _session.markDocumentDirty();
    notifyListeners();
  }

  static const _clipboardHtml = 'text/html';

  Future<EditorClipboardPayload> readClipboard() async {
    final plain = await Clipboard.getData(Clipboard.kTextPlain);
    final html = await Clipboard.getData(_clipboardHtml);
    return EditorClipboardPayload(
      plainText: plain?.text,
      html: html?.text,
      docxBytes: _lastClipboardDocx,
    );
  }

  Future<void> paste({bool plainText = false}) async {
    await pastePayload(await readClipboard(), plainText: plainText);
  }

  Future<void> showPasteSpecialDialog(BuildContext context) async {
    final payload = await readClipboard();
    final mode = await PasteSpecialDialog.show(
      context,
      hasFormattedContent: payload.hasFormattedContent,
    );
    if (mode == null) return;
    await pastePayload(payload, plainText: mode == PasteSpecialMode.plainText);
  }

  Future<void> pastePayload(EditorClipboardPayload payload, {required bool plainText}) async {
    if (!_host.isConnected) return;
    if (_selection.hasGlyphSelection) await _selection.deleteGlyphSelection();
    final runId = _selection.defaultRunId();
    if (runId == null) return;

    final pasted = !plainText && await _tryPasteFormatted(runId, payload) ||
        await _tryPastePlain(runId, payload.plainText);
    if (!pasted) return;

    _selection.collapseToCaret();
    _formatting.syncFromCaret();
    _session.markDocumentDirty();
    notifyListeners();
  }

  Future<void> deleteSelection() async {
    if (hasSelectedObject) {
      await deleteSelectedObject();
      return;
    }
    if (!canCutOrCopy) return;
    await _selection.deleteGlyphSelection();
    _session.markDocumentDirty();
    notifyListeners();
  }

  // ── Editing ───────────────────────────────────────────────────────────────
  void insertCharacter(String char) {
    if (_host.isConnected) {
      unawaited(insertGlyphCharacter(char));
      return;
    }
  }

  void deleteBackward() {
    if (_host.isConnected) unawaited(deleteGlyphBackward());
  }

  void deleteForward() {
    if (_host.isConnected) unawaited(deleteGlyphForward());
  }

  Future<void> _runGlyphMutation(Future<void> Function() body) {
    final released = Completer<void>();
    final previous = _glyphMutationTail;
    _glyphMutationTail = released.future;
    return previous.then((_) => body()).whenComplete(released.complete);
  }

  bool _isNavigationInput(EditorInputEvent event) {
    switch (event.kind) {
      case EditorInputKind.arrowLeft:
      case EditorInputKind.arrowRight:
      case EditorInputKind.arrowUp:
      case EditorInputKind.arrowDown:
      case EditorInputKind.wordLeft:
      case EditorInputKind.wordRight:
      case EditorInputKind.paragraphUp:
      case EditorInputKind.paragraphDown:
      case EditorInputKind.lineStart:
      case EditorInputKind.lineEnd:
      case EditorInputKind.documentStart:
      case EditorInputKind.documentEnd:
      case EditorInputKind.pageUp:
      case EditorInputKind.pageDown:
        return true;
      default:
        return false;
    }
  }

  Future<void> insertGlyphCharacter(String char) async {
    if (char == '\n' || char == '\r') return;
    // Reject C0 controls and DEL (U+007F) — Delete keys must not insert text.
    final code = char.codeUnitAt(0);
    if (char != '\t' && (code < 0x20 || code == 0x7f)) return;
    await _runGlyphMutation(() async {
      if (_selection.hasGlyphSelection) {
        await _selection.deleteGlyphSelection();
      }
      await _insertGlyphText(char);
    });
  }

  /// Shift+Enter — a manual line break stays inside the paragraph. `\n` in run
  /// text is the engine's line break: layout treats it as a mandatory break and
  /// the DOCX exporter writes it as `<w:br/>`, matching Word.
  Future<void> insertGlyphLineBreak() => _runGlyphMutation(() async {
        if (_selection.hasGlyphSelection) {
          await _selection.deleteGlyphSelection();
        }
        await _insertGlyphText('\n');
      });

  Future<void> _insertGlyphText(String char) async {
    if (_host.engine == null) return;
    if (hasSelectedDiagram) {
      final entered = await enterSelectedShapeTextEdit();
      if (!entered) return;
    }
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    final offset = _selection.caretOffset;
    // Advance the logical caret synchronously so overlapping keystrokes
    // capture sequential offsets before any worker await.
    final optimistic = offset + char.runes.length;
    _selection.afterInsert(runId, optimistic);
    _session.markDocumentDirty();
    notifyListeners();
    final edit = _host.performNativeEdit(
      () => _host.engine!.tryInsertTextAsync(runId, offset, char),
      dirtyPage: _selection.caretPage,
    );
    if (!await edit) {
      _rollbackCaret(runId, from: optimistic, to: offset);
    } else if (_selection.caretRunId == runId &&
        _selection.caretOffset == optimistic) {
      await _host.ensureLayoutReady();
      final pageBefore = _selection.caretPage;
      final yBefore = _selection.caretGeometry?.y;
      _selection.syncCaretGeometry();
      final yAfter = _selection.caretGeometry?.y;
      final movedVertically = _selection.caretPage != pageBefore ||
          yAfter == null ||
          yBefore == null ||
          (yAfter - yBefore).abs() > 0.5;
      if (movedVertically) {
        ensureCaretVisible(notify: false);
      }
    }
    notifyListeners();
  }

  /// Undoes an optimistic caret advance, but only when nothing has moved the
  /// caret since — a later keystroke's position must win over an earlier
  /// failure's stale offset.
  void _rollbackCaret(String runId, {required int from, required int to}) {
    if (_selection.caretRunId != runId || _selection.caretOffset != from) return;
    _selection.afterInsert(runId, to);
    notifyListeners();
  }

  Future<void> insertGlyphParagraphBreak() async {
    if (_host.engine == null) return;
    await _runGlyphMutation(() async {
      if (hasSelectedDiagram) {
        final entered = await enterSelectedShapeTextEdit();
        if (!entered) return;
      }
      if (_selection.hasGlyphSelection) {
        await _selection.deleteGlyphSelection();
      }
      final runId = _selection.defaultRunId();
      if (runId == null) return;
      // Capture the logical caret *before* any geometry sync. Geometry refresh
      // must never rewrite the split offset (stale hit-tests used to snap to 0,
      // so Enter moved the whole line and looked like a copy).
      final splitOffset = _selection.caretOffset;
      HitTestResult? newCaret;
      // Full refresh: Enter may soft-paginate onto a new page; dirty-page-only
      // refresh leaves pageCount stale so cross-page caret sync cannot see it.
      final edit = _host.performNativeEdit(() async {
        newCaret = await _host.engine!.splitParagraphAsync(runId, splitOffset);
        return newCaret != null;
      }, full: true);
      await edit;
      await _host.ensureLayoutReady();

      if (newCaret != null) {
        _selection.setCaret(newCaret!.runId, newCaret!.charOffset);
        _selection.syncCaretGeometry();
      }
      _selection.collapseToCaret();

      ensureCaretVisible(notify: false);
      _session.markDocumentDirty();
      notifyListeners();
    });
  }

  Future<void> deleteGlyphBackward() async {
    if (_host.engine == null) return;
    await _runGlyphMutation(() async {
      if (hasSelectedObject) {
        if (_preferShapeTextDeleteOverObjectDelete()) {
          clearDiagramSelection();
        } else {
          await deleteSelectedObject();
          return;
        }
      }
      if (_selection.hasGlyphSelection) {
        await _selection.deleteGlyphSelection();
        _session.markDocumentDirty();
        return;
      }
      final runId = _selection.defaultRunId();
      if (runId == null) return;
      if (_selection.caretOffset > 0) {
        final off = _selection.caretOffset;
        final edit = _host.performNativeEdit(
          () => _host.engine!.deleteRangeAsync(runId, off - 1, off),
          dirtyPage: _selection.caretPage,
          full: true,
        );
        final optimistic = off - 1;
        _selection.afterInsert(runId, optimistic);
        _session.markDocumentDirty();
        notifyListeners();
        if (!await edit) {
          _rollbackCaret(runId, from: optimistic, to: off);
        } else if (_selection.caretRunId == runId &&
            _selection.caretOffset == optimistic) {
          _selection.syncCaretGeometry();
        }
        notifyListeners();
      }
    });
  }

  /// True when Backspace/Delete should edit shape body text rather than remove
  /// the selected object (caret still inside the frame).
  bool _preferShapeTextDeleteOverObjectDelete() {
    if (!hasSelectedDiagram) return false;
    final rect = _selectedDiagramRect;
    final geom = _selection.caretGeometry;
    if (rect == null || geom == null) return false;
    final point = Offset(geom.x, geom.y + geom.height * 0.5);
    return rect.inflate(4).contains(point);
  }

  /// Ctrl+Backspace / Ctrl+Delete — remove the adjacent word in one undo step.
  Future<void> deleteGlyphWord({required bool forward}) async {
    if (_host.engine == null) return;
    if (hasSelectedObject || _selection.hasGlyphSelection) {
      await (forward ? deleteGlyphForward() : deleteGlyphBackward());
      return;
    }
    final runId = _selection.defaultRunId();
    final range = _selection.wordDeleteRange(direction: forward ? 1 : -1);
    if (runId == null || range == null) {
      // At the run edge there is no word to take, so fall back to the plain
      // character delete, which knows how to cross runs.
      await (forward ? deleteGlyphForward() : deleteGlyphBackward());
      return;
    }
    await _runGlyphMutation(() async {
      final before = _selection.caretOffset;
      final edit = _host.performNativeEdit(
        () => _host.engine!.deleteRangeAsync(runId, range.$1, range.$2),
        dirtyPage: _selection.caretPage,
      );
      _selection.afterInsert(runId, range.$1);
      _session.markDocumentDirty();
      notifyListeners();
      if (!await edit) {
        _rollbackCaret(runId, from: range.$1, to: before);
      } else {
        _selection.syncCaretGeometry();
      }
      notifyListeners();
    });
  }

  Future<void> deleteGlyphForward() async {
    if (_host.engine == null) return;
    await _runGlyphMutation(() async {
      if (hasSelectedObject) {
        if (_preferShapeTextDeleteOverObjectDelete()) {
          clearDiagramSelection();
        } else {
          await deleteSelectedObject();
          return;
        }
      }
      if (_selection.hasGlyphSelection) {
        await _selection.deleteGlyphSelection();
        _session.markDocumentDirty();
        return;
      }
      final runId = _selection.defaultRunId();
      if (runId == null) return;
      final off = _selection.caretOffset;
      final edit = _host.performNativeEdit(
        () => _host.engine!.deleteRangeAsync(runId, off, off + 1),
        dirtyPage: _selection.caretPage,
        full: true,
      );
      _selection.collapseToCaret();
      notifyListeners();
      if (await edit) {
        _session.markDocumentDirty();
        _selection.syncCaretGeometry();
      }
      notifyListeners();
    });
  }

  Future<void> moveGlyphSelectionTo(int pageIndex, double x, double y) async {
    if (_host.engine == null || !_selection.hasGlyphSelection) return;
    final text = selectedText;
    if (text.isEmpty) return;
    await _selection.deleteGlyphSelection();
    final drop = _host.engine!.hitTestPage(pageIndex, x, y);
    if (drop == null) return;
    final edit = _host.performNativeEdit(
      () => _host.engine!.tryInsertTextAsync(drop.runId, drop.charOffset, text),
      dirtyPage: _selection.caretPage,
    );
    if (!await edit) return;
    _selection.afterInsert(drop.runId, drop.charOffset + text.length);
    _selection.syncCaretGeometry();
    _formatting.syncFromCaret();
    _session.markDocumentDirty();
    notifyListeners();
  }

  // ── Document lifecycle (delegated) ────────────────────────────────────────
  Future<void> newDocument() => _session.newDocument();

  /// Load a built-in starter `.docx` as an untitled document (F24.S1)
  /// and apply its bound Design gallery theme (F24.S2).
  Future<void> newFromTemplate(DocumentTemplateSpec template) async {
    final data = await rootBundle.load(template.assetPath);
    final bytes = data.buffer.asUint8List();
    await _session.newFromTemplate(
      bytes: bytes,
      templateTitle: template.title,
      assetPath: template.assetPath,
    );
    await _session.applyEngineStyle(
      () => _host.engine!.applyDocumentThemeAsync(themeName: template.themeName),
      'New from template: ${template.title} (${template.themeName})',
    );
    _documentThemeName = template.themeName;
    notifyListeners();
  }

  /// Open a user-saved template as untitled and re-apply its theme (F24.S3).
  Future<void> newFromUserTemplate(UserTemplateEntry entry) async {
    final bytes = await _session.readUserTemplateBytes(entry.id);
    if (bytes == null || bytes.isEmpty) {
      _session.setStatusText('Template not found: ${entry.title}');
      notifyListeners();
      return;
    }
    await _session.newFromTemplate(
      bytes: bytes,
      templateTitle: entry.title,
      assetPath: 'user-template://${entry.id}.docx',
    );
    await _session.applyEngineStyle(
      () => _host.engine!.applyDocumentThemeAsync(themeName: entry.themeName),
      'New from template: ${entry.title} (${entry.themeName})',
    );
    _documentThemeName = entry.themeName;
    notifyListeners();
  }

  /// Show the New from Template chooser and open the selection (F24.S1 / F24.S3).
  Future<void> openNewFromTemplateDialog(BuildContext context) async {
    final selection = await NewFromTemplateDialog.show(
      context,
      userTemplates: _session.userTemplates,
    );
    if (selection == null) return;
    if (selection.spec != null) {
      await newFromTemplate(selection.spec!);
    } else if (selection.user != null) {
      await newFromUserTemplate(selection.user!);
    }
  }

  /// Snapshot the open document into My Templates (F24.S3).
  Future<void> openSaveAsTemplateDialog(BuildContext context) async {
    final suggested = documentTitle.trim().isEmpty || documentTitle == 'Document1'
        ? 'My Template'
        : documentTitle;
    final title = await SaveAsTemplateDialog.show(
      context,
      initialTitle: suggested,
    );
    if (title == null) return;
    await _session.saveAsTemplate(
      title: title,
      themeName: _documentThemeName,
    );
    notifyListeners();
  }
  Future<void> openDocument() => _session.openDocument();
  Future<void> openDocumentFromPath(String path) => _session.openDocumentFromPath(path);
  Future<void> openRecentDocument(String path) => _session.openRecentDocument(path);

  /// Install UI password prompt for encrypted document opens (F22.S1).
  void setPasswordPrompt(PasswordPromptCallback? prompt) =>
      _session.setPasswordPrompt(prompt);
  Future<void> saveDocument() => _session.saveDocument();
  Future<void> saveDocumentAs({required String extension}) =>
      _session.saveDocumentAs(extension: extension);
  Future<bool> saveDocumentToPath(String path, {String? formatExtension}) =>
      _session.saveDocumentToPath(path, formatExtension: formatExtension);

  bool get encryptionPasswordSet => _session.encryptionPasswordSet;

  /// Show protect dialog and set encryption password for DOCX saves (F22.S2).
  Future<void> protectWithPassword(BuildContext context) async {
    final password = await ProtectPasswordDialog.show(context);
    if (password == null) {
      _session.setStatusText('Protect cancelled');
      return;
    }
    _session.protectWithPassword(password);
  }

  Future<void> removePasswordProtection() async {
    _session.removePasswordProtection();
  }

  /// Show Inspect Document dialog and remove selected categories (F22.S3).
  Future<void> inspectDocument(BuildContext context) async {
    final json = _session.fetchDocumentInspectJson() ?? '[]';
    List<DocumentInspectFinding> findings = const [];
    try {
      final decoded = jsonDecode(json);
      if (decoded is List) {
        findings = decoded
            .whereType<Map>()
            .map((e) =>
                DocumentInspectFinding.fromJson(Map<String, dynamic>.from(e)))
            .toList();
      }
    } catch (_) {
      findings = const [];
    }
    if (!context.mounted) return;
    final removal = await DocumentInspectorDialog.show(context, findings);
    if (removal == null || !removal.any) {
      if (removal == null) {
        _session.setStatusText('Inspect cancelled');
      }
      return;
    }
    _session.removeInspectFindings(
      comments: removal.comments,
      metadata: removal.metadata,
      hiddenText: removal.hiddenText,
    );
  }

  /// Show Digital Signatures dialog (sign / verify / clear) (F22.S4).
  Future<void> manageDigitalSignatures(BuildContext context) async {
    List<DigitalSignatureInfo> parseSignatures(String? json) {
      try {
        final decoded = jsonDecode(json ?? '[]');
        if (decoded is! List) return const [];
        return decoded
            .whereType<Map>()
            .map((e) =>
                DigitalSignatureInfo.fromJson(Map<String, dynamic>.from(e)))
            .toList();
      } catch (_) {
        return const [];
      }
    }

    List<SignatureVerificationInfo> parseVerifications(String? json) {
      try {
        final decoded = jsonDecode(json ?? '[]');
        if (decoded is! List) return const [];
        return decoded
            .whereType<Map>()
            .map((e) => SignatureVerificationInfo.fromJson(
                  Map<String, dynamic>.from(e),
                ))
            .toList();
      } catch (_) {
        return const [];
      }
    }

    final signatures =
        parseSignatures(_session.fetchDigitalSignaturesJson());
    final verifications =
        parseVerifications(_session.verifyDigitalSignaturesJson());
    if (!context.mounted) return;
    final result = await DigitalSignatureDialog.show(
      context,
      signatures: signatures,
      verifications: verifications,
    );
    if (result == null) return;
    if (result == 'clear') {
      _session.clearDigitalSignatures();
      return;
    }
    if (result is DigitalSignatureSignRequest) {
      _session.signDocument(
        name: result.name,
        email: result.email,
        organization: result.organization,
      );
    }
  }

  Future<void> exportPdf() => _session.exportPdf();
  Future<bool> exportPdfToPath(String path) => _session.exportPdfToPath(path);

  /// File→Print: scale/margins/scope settings, then OS print dialog (F25.S1–S3).
  ///
  /// When [layout] is omitted and [context] is mounted, shows [PrintSettingsDialog].
  /// Pass [layout] (or omit [context]) to skip the settings UI — used by tests.
  Future<PrintDialogResult> printDocument({
    BuildContext? context,
    PrintLayoutSettings? layout,
  }) async {
    var settings = layout;
    if (settings == null && context != null && context.mounted) {
      settings = await PrintSettingsDialog.show(
        context,
        selectionAvailable: hasGlyphSelection,
      );
      if (settings == null) {
        _session.setStatusText('Print cancelled');
        notifyListeners();
        return const PrintDialogResult(
          outcome: PrintDialogOutcome.cancelled,
        );
      }
    }
    settings ??= PrintLayoutSettings.defaults;

    DocRange? selection;
    if (settings.scope == PrintScope.selection) {
      if (!hasGlyphSelection || _selection.selection == null) {
        _session.setStatusText('Print failed: no selection');
        notifyListeners();
        return const PrintDialogResult(
          outcome: PrintDialogOutcome.failed,
          message: 'no selection',
        );
      }
      selection = _selection.selection;
    }

    final bytes = _session.printPdfBytes(settings, selection);
    if (bytes == null || bytes.isEmpty || !isPdfHeader(bytes)) {
      _session.setStatusText('Print failed: could not build PDF');
      notifyListeners();
      return const PrintDialogResult(
        outcome: PrintDialogOutcome.failed,
        message: 'could not build PDF',
      );
    }
    final result = await _printHost.presentPrintDialog(
      pdfBytes: bytes,
      jobName: documentTitle,
      attributes: settings.toPlatformAttributes(),
    );
    switch (result.outcome) {
      case PrintDialogOutcome.presented:
        _session.setStatusText(
          selection != null
              ? 'Print dialog opened (selection)'
              : 'Print dialog opened',
        );
      case PrintDialogOutcome.cancelled:
        _session.setStatusText('Print cancelled');
      case PrintDialogOutcome.fallbackSaved:
        _session.setStatusText(
          result.message ?? 'Print PDF saved for printing',
        );
      case PrintDialogOutcome.unsupported:
        _session.setStatusText('Print not supported on this platform');
      case PrintDialogOutcome.failed:
        _session.setStatusText(result.message ?? 'Print failed');
    }
    notifyListeners();
    return result;
  }

  /// Share the document via the OS share sheet (Mail, Messages, WhatsApp, etc.).
  ///
  /// Prefers a temporary `.docx` attachment when the engine can serialize one;
  /// otherwise shares plain text. Does not use cloud share links.
  Future<DocumentShareResult> shareWithApps() async {
    final title = documentTitle.trim().isEmpty ? 'Document' : documentTitle.trim();
    final text = documentText;
    String? filePath;
    String? mimeType;

    try {
      final docx = _host.engine?.saveDocumentAsBytes('docx');
      if (docx != null && docx.isNotEmpty) {
        filePath = await _shareTempWriter(
          fileName: '$title.docx',
          bytes: docx,
        );
        if (filePath != null) {
          mimeType =
              'application/vnd.openxmlformats-officedocument.wordprocessingml.document';
        }
      } else if (text.trim().isNotEmpty) {
        filePath = await _shareTempWriter(
          fileName: '$title.txt',
          bytes: Uint8List.fromList(utf8.encode(text)),
        );
        if (filePath != null) {
          mimeType = 'text/plain';
        }
      }
    } catch (_) {
      // Fall through to text-only share.
    }

    if ((filePath == null || filePath.isEmpty) && text.trim().isEmpty) {
      _session.setStatusText('Nothing to share');
      notifyListeners();
      return const DocumentShareResult(
        outcome: DocumentShareOutcome.failed,
        message: 'Nothing to share',
      );
    }

    final result = await _shareHost.share(
      text: text,
      subject: title,
      filePath: filePath,
      mimeType: mimeType,
    );
    switch (result.outcome) {
      case DocumentShareOutcome.presented:
        _session.setStatusText(result.message ?? 'Share sheet opened');
      case DocumentShareOutcome.cancelled:
        _session.setStatusText('Share cancelled');
      case DocumentShareOutcome.unsupported:
        _session.setStatusText(
          result.message ?? 'Share not supported on this device',
        );
      case DocumentShareOutcome.failed:
        _session.setStatusText(result.message ?? 'Share failed');
    }
    notifyListeners();
    return result;
  }

  Future<void> undo() => _session.undo();
  Future<void> redo() => _session.redo();
  Future<void> setAutosaveInterval(Duration d) => _session.setAutosaveInterval(d);
  Future<void> performAutosave() => _session.performAutosave();
  Future<bool> tryRecoverAutosave() => _session.tryRecoverAutosave();
  void toggleTrackChanges() => _session.toggleTrackChanges();
  void acceptAllRevisions() => _session.acceptAllRevisions();
  void rejectAllRevisions() => _session.rejectAllRevisions();
  void acceptRevisionAtCaret() => _session.acceptRevisionAtCaret();
  void rejectRevisionAtCaret() => _session.rejectRevisionAtCaret();
  void gotoNextRevision() => _session.gotoNextRevision();
  void gotoPreviousRevision() => _session.gotoPreviousRevision();
  Future<void> spellCheckDocument() => _session.spellCheckDocument();
  Future<void> grammarCheckDocument() => _session.grammarCheckDocument();
  Future<void> proofDocument([BuildContext? context]) async {
    await _session.proofDocument();
    if (context != null && context.mounted) {
      final issues = _session.spellIssues;
      if (issues.any((issue) => issue.suggestions.isNotEmpty)) {
        await SpellSuggestionsDialog.show(
          context,
          issues,
          onApplySuggestion: applySpellSuggestion,
        );
      }
    }
    notifyListeners();
  }

  Future<void> applySpellSuggestion(SpellIssue issue, String suggestion) async {
    if (_host.engine == null) return;
    if (issue.start != null && issue.end != null) {
      final edit = _host.performNativeEdit(
        () => _host.engine!.applySpellReplacementAsync(
          plainStart: issue.start!,
          plainEnd: issue.end!,
          replacement: suggestion,
        ),
        full: true,
      );
      if (await edit) {
        _session.setStatusText('Spelling correction applied');
        _session.markDocumentDirty();
        notifyListeners();
      }
      return;
    }
    final range = selection;
    if (range != null) {
      final (start, end) = range.normalized();
      if (start.runId == end.runId) {
        await applyAiTextSuggestion(
          runId: start.runId,
          start: start.offset,
          end: end.offset,
          text: suggestion,
        );
      }
    }
  }

  Future<CommentThreadList> loadCommentThreads() async {
    final json = _host.engine?.getCommentsJson();
    return CommentThreadList.parse(json ?? '[]');
  }

  Future<void> replyToComment(int commentId, String bodyText) async {
    if (_host.engine == null) return;
    await _host.performNativeEdit(
      () => _host.engine!.replyToCommentAsync(
        commentId: commentId,
        bodyText: bodyText,
      ),
      full: true,
    );
    _session.setStatusText('Reply added');
    notifyListeners();
  }

  Future<void> resolveComment(int commentId, {required bool resolved}) async {
    if (_host.engine == null) return;
    await _host.performNativeEdit(
      () => _host.engine!.resolveCommentAsync(
        commentId: commentId,
        resolved: resolved,
      ),
      full: true,
    );
    _session.setStatusText(resolved ? 'Comment resolved' : 'Comment reopened');
    notifyListeners();
  }

  Future<void> showCommentsPane(BuildContext context) async {
    await CommentsPane.show(context, this);
    notifyListeners();
  }
  void compareWithText(String otherText) => _session.compareWithText(otherText);

  /// Review → Compare: pick another document and diff against the current one.
  Future<void> compareWithDocumentPicker(BuildContext context) async {
    if (!_host.isConnected || _host.engine == null) return;
    try {
      final picked = await pickDocumentFile(
        dialogTitle: 'Compare with document',
      );
      if (picked == null) {
        _session.setStatusText('Compare cancelled');
        notifyListeners();
        return;
      }
      final otherText = DocumentReader.extractText(
        picked.bytes,
        path: picked.path,
      );
      final currentText = documentText;
      compareWithText(otherText);
      final diff = compareDocumentLines(currentText, otherText);
      // Prefer the engine summary when present; keep Dart counts as fallback.
      _session.setCompareSummary(diff.summary);
      _session.setStatusText('Compare complete (${diff.summary})');
      notifyListeners();
      if (!context.mounted) return;
      await CompareResultsDialog.show(
        context,
        result: diff,
        otherLabel: picked.name,
      );
    } catch (e) {
      _session.setStatusText('Compare failed: $e');
      notifyListeners();
      if (context.mounted) {
        ScaffoldMessenger.maybeOf(context)?.showSnackBar(
          SnackBar(content: Text('Compare failed: $e')),
        );
      }
    }
  }
  void toggleRestrictEditing() => _session.toggleRestrictEditing();
  void clearInfoMessage() => _session.clearInfoMessage();

  Future<void> insertTable({int rows = 3, int cols = 3}) async {
    if (!_host.isConnected) return;
    final safeRows = rows.clamp(1, 63);
    final safeCols = cols.clamp(1, 63);
    await _session.applyEngineStyle(
      () => _host.engine!.insertTableBlockAsync(
        safeRows,
        safeCols,
        caretRunId: _selection.defaultRunId(),
      ),
      'Table inserted ($safeRows×$safeCols)',
      full: true,
    );
    await _placeCaretInLatestShape();
    notifyListeners();
  }

  /// After insert, put the caret in the new table/shape so typing works
  /// without an extra click (ribbon buttons otherwise keep keyboard focus).
  Future<void> _placeCaretInLatestShape() async {
    if (_host.engine == null) {
      requestEditorFocus();
      return;
    }
    await _host.ensureLayoutReady();
    ShapeBounds? latest;
    var latestPage = _view.currentPage;
    final pages = <int>[_view.currentPage];
    for (var page = 0; page < pageCount; page++) {
      if (page != _view.currentPage) pages.add(page);
    }
    for (final page in pages) {
      final snap = DisplayListSnapshot.fromBytes(_host.displayListForPage(page));
      if (snap.shapeIds.isEmpty) continue;
      final index = snap.shapeIds.length - 1;
      if (snap.shapeRects.length < (index + 1) * 4) continue;
      latest = ShapeBounds(
        shapeId: snap.shapeIds[index],
        index: index,
        rect: Rect.fromLTWH(
          snap.shapeRects[index * 4],
          snap.shapeRects[index * 4 + 1],
          snap.shapeRects[index * 4 + 2],
          snap.shapeRects[index * 4 + 3],
        ),
      );
      latestPage = page;
      break;
    }
    if (latest == null) {
      requestEditorFocusAfterOverlay();
      return;
    }
    selectDiagram(latestPage, latest);
    await enterSelectedShapeTextEdit();
    requestEditorFocusAfterOverlay();
  }

  static const int shapeRectangle = 0;
  static const int shapeLine = 1;
  static const int shapeEllipse = 2;

  Future<void> insertShape(int shapeType) async {
    if (!_host.isConnected) return;
    final label = switch (shapeType) {
      shapeLine => 'Line inserted',
      shapeEllipse => 'Ellipse inserted',
      _ => 'Rectangle inserted',
    };
    await _session.applyEngineStyle(
      () => _host.engine!.insertShapeBlockAsync(
        shapeType,
        caretRunId: _selection.defaultRunId(),
      ),
      label,
      full: true,
    );
    if (shapeType != shapeLine) {
      await _placeCaretInLatestShape();
    } else {
      requestEditorFocus();
    }
    notifyListeners();
  }

  Future<void> insertTextBox() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertTextBoxAsync(
        caretRunId: _selection.defaultRunId(),
      ),
      'Text box inserted',
      full: true,
    );
    await _placeCaretInLatestShape();
    notifyListeners();
  }

  Future<void> insertWordArt(String text) async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertWordArtAsync(
        text,
        caretRunId: _selection.defaultRunId(),
      ),
      'WordArt inserted',
      full: true,
    );
    await _placeCaretInLatestShape();
    notifyListeners();
  }

  /// SmartArt kinds matching [tw_model::DiagramKind].
  static const int smartArtProcess = 0;
  static const int smartArtHierarchy = 1;
  static const int smartArtCycle = 2;

  Future<void> insertSmartArt({int diagramType = smartArtProcess}) async {
    if (!_host.isConnected) return;
    final label = switch (diagramType) {
      smartArtHierarchy => 'Hierarchy SmartArt inserted',
      smartArtCycle => 'Cycle SmartArt inserted',
      _ => 'Process SmartArt inserted',
    };
    await _session.applyEngineStyle(
      () => _host.engine!.insertDiagramAsync(
        diagramType: diagramType,
        caretRunId: _selection.defaultRunId(),
      ),
      label,
      full: true,
    );
    await _placeCaretInLatestShape();
    notifyListeners();
  }

  /// Chart kinds matching [tw_model::ChartKind] / Insert Chart gallery.
  static const int chartColumn = 0;
  static const int chartBar = 1;
  static const int chartLine = 2;
  static const int chartPie = 3;

  Future<void> insertChart({int chartType = chartColumn}) async {
    if (!_host.isConnected) return;
    final label = switch (chartType) {
      chartBar => 'Bar chart inserted',
      chartLine => 'Line chart inserted',
      chartPie => 'Pie chart inserted',
      _ => 'Column chart inserted',
    };
    await _session.applyEngineStyle(
      () => _host.engine!.insertChartAsync(
        chartType: chartType,
        caretRunId: _selection.defaultRunId(),
      ),
      label,
      full: true,
    );
    final chartId = _host.engine?.latestChartId();
    if (chartId != null) {
      selectDiagramById(chartId);
    }
    notifyListeners();
  }

  /// Selects a shape/chart by id, resolving its page rect from display lists.
  void selectDiagramById(String shapeId) {
    clearImageSelection();
    _selection.collapseToCaret();
    for (var page = 0; page < pageCount; page++) {
      final snap = DisplayListSnapshot.fromBytes(_host.displayListForPage(page));
      final index = snap.shapeIds.indexOf(shapeId);
      if (index < 0 || snap.shapeRects.length < (index + 1) * 4) continue;
      final rect = Rect.fromLTWH(
        snap.shapeRects[index * 4],
        snap.shapeRects[index * 4 + 1],
        snap.shapeRects[index * 4 + 2],
        snap.shapeRects[index * 4 + 3],
      );
      selectDiagram(page, ShapeBounds(shapeId: shapeId, index: index, rect: rect));
      return;
    }
    _selectedDiagramId = shapeId;
    _selectedDiagramPage = _view.currentPage;
    _selectedDiagramRect = null;
    notifyListeners();
  }

  /// Opens Edit Data for [shapeId] (or the selected / latest chart).
  Future<bool> editChartData(
    BuildContext context, {
    String? shapeId,
  }) async {
    if (!_host.isConnected || _host.engine == null) return false;
    final id = shapeId ?? _selectedDiagramId ?? _host.engine!.latestChartId();
    if (id == null) return false;

    final json = _host.engine!.fetchChartDataJson(id);
    if (json == null || json.isEmpty) return false;

    ChartDataModel initial;
    try {
      initial = ChartDataModel.fromJson(
        jsonDecode(json) as Map<String, dynamic>,
      );
    } catch (_) {
      return false;
    }

    final edited = await ChartDataDialog.show(context, initial: initial);
    if (edited == null) return true; // dismissed — still a chart interaction

    final ok = await _host.performNativeEdit(
      () => _host.engine!.setChartDataAsync(id, edited.toJson()),
      full: true,
    );
    if (ok) {
      _session.setStatusText('Chart data updated');
      _session.markDocumentDirty();
      selectDiagramById(id);
      notifyListeners();
    } else {
      final err = _host.engine?.getLastError();
      _session.setStatusText(
        (err != null && err.isNotEmpty) ? err : 'Chart data update failed',
      );
    }
    return ok;
  }

  /// Double-click / Edit Data entry: open chart sheet when the hit is a chart.
  Future<bool> editChartDataAt(
    BuildContext context,
    int pageIndex,
    Offset point,
    DisplayListSnapshot snapshot,
  ) async {
    final hit = hitTestShape(snapshot, point);
    if (hit == null) return false;
    selectDiagram(pageIndex, hit);
    return editChartData(context, shapeId: hit.shapeId);
  }

  /// Opens Insert Equation from the ribbon (F14.S3).
  Future<void> insertEquation(BuildContext context) async {
    if (!_host.isConnected) return;
    final edited = await EquationDialog.show(
      context,
      initial: EquationModel.plain(text: 'x', display: true),
    );
    if (edited == null) return;
    await _applyEquationInsert(edited);
  }

  Future<void> _applyEquationInsert(EquationModel model) async {
    final xml = model.toOmml();
    final runId = _selection.defaultRunId();
    if (model.display) {
      await _session.applyEngineStyle(
        () => _host.engine!.insertOfficeMathDisplayAsync(
          caretRunId: runId,
          xml: xml,
        ),
        'Equation inserted',
        full: true,
      );
    } else {
      if (runId == null) return;
      await _session.applyEngineStyle(
        () => _host.engine!.insertOfficeMathAsync(
          runId: runId,
          offset: _selection.caretOffset,
          xml: xml,
        ),
        'Equation inserted',
        full: true,
      );
    }
    final mathRunId = _host.engine?.latestOfficeMathRunId();
    if (mathRunId != null) {
      _selection.setCaret(mathRunId, 1, page: _selection.caretPage);
      _selection.syncCaretGeometry();
    }
    notifyListeners();
  }

  /// Opens Insert Symbol from the ribbon (F15.S1).
  Future<void> insertSymbol(BuildContext context) async {
    if (!_host.isConnected) return;
    final symbol = await SymbolDialog.show(context, recentStore: _recentSymbols);
    if (symbol == null || symbol.isEmpty) return;
    await insertSymbolCharacter(symbol);
  }

  /// Inserts [symbol] at the caret via `InsertText` (F15.S1).
  Future<void> insertSymbolCharacter(String symbol) async {
    if (!_host.isConnected || symbol.isEmpty) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    final offset = _selection.caretOffset;
    final edit = _host.performNativeEdit(
      () => _host.engine!.tryInsertTextAsync(runId, offset, symbol),
      dirtyPage: _selection.caretPage,
    );
    final optimistic = offset + symbol.length;
    _selection.afterInsert(runId, optimistic);
    _session.markDocumentDirty();
    notifyListeners();
    if (!await edit) {
      _rollbackCaret(runId, from: optimistic, to: offset);
      final err = _host.engine?.getLastError();
      _session.setStatusText(
        (err != null && err.isNotEmpty) ? err : 'Symbol insert failed',
      );
    } else {
      _session.setStatusText('Symbol inserted');
      _recentSymbols.recordCharacter(symbol);
      await _sessionStore?.saveRecentSymbolIds(_recentSymbols.exportIds());
      if (_selection.caretRunId == runId && _selection.caretOffset == optimistic) {
        _selection.syncCaretGeometry();
      }
    }
    notifyListeners();
  }

  /// Edit an existing equation run (F14.S3).
  Future<bool> editEquation(
    BuildContext context, {
    String? runId,
  }) async {
    if (!_host.isConnected || _host.engine == null) return false;
    final id = runId ?? _selection.caretRunId ?? _host.engine!.latestOfficeMathRunId();
    if (id == null) return false;

    final xml = _host.engine!.fetchOfficeMathXml(id);
    if (xml == null || xml.isEmpty) return false;

    final initial = EquationOmml.fromOmml(xml) ?? EquationModel.plain(text: 'x');
    final edited = await EquationDialog.show(context, initial: initial);
    if (edited == null) return true;

    final ok = await _host.performNativeEdit(
      () => _host.engine!.setOfficeMathAsync(id, edited.toOmml()),
      full: true,
    );
    if (ok) {
      _session.setStatusText('Equation updated');
      _session.markDocumentDirty();
      _selection.setCaret(id, 1, page: _selection.caretPage);
      _selection.syncCaretGeometry();
      notifyListeners();
    }
    return ok;
  }

  Future<void> deleteTableRow() async {
    if (!await _prepareTableCellEdit()) return;
    await _session.applyEngineStyle(
      () => _host.engine!.deleteTableRowAsync(
        caretRunId: _selection.defaultRunId(),
      ),
      'Table row deleted',
      full: true,
    );
    notifyListeners();
  }

  Future<void> deleteTableColumn() async {
    if (!await _prepareTableCellEdit()) return;
    await _session.applyEngineStyle(
      () => _host.engine!.deleteTableColumnAsync(
        caretRunId: _selection.defaultRunId(),
      ),
      'Table column deleted',
      full: true,
    );
    notifyListeners();
  }

  Future<void> mergeTableCells() async {
    if (!await _prepareTableCellEdit()) return;
    await _session.applyEngineStyle(
      () => _host.engine!.mergeTableCellsAsync(
        caretRunId: _selection.defaultRunId(),
      ),
      'Cells merged',
      full: true,
    );
    notifyListeners();
  }

  Future<void> splitTableCell() async {
    if (!await _prepareTableCellEdit()) return;
    await _session.applyEngineStyle(
      () => _host.engine!.splitTableCellAsync(
        caretRunId: _selection.defaultRunId(),
      ),
      'Cell split',
      full: true,
    );
    notifyListeners();
  }

  Future<void> showTableDesignDialog(BuildContext context) async {
    final values = await TableDesignDialog.show(
      context,
      initial: const TableDesignValues(columnWidth: 100),
    );
    if (values == null) return;
    await applyTableDesign(values);
  }

  Future<void> applyTableDesign(TableDesignValues values) async {
    if (!await _prepareTableCellEdit()) return;
    final caret = _selection.defaultRunId();

    if (values.autofitToWindow) {
      await _session.applyEngineStyle(
        () => _host.engine!.autofitTableAsync(caretRunId: caret),
        'Table autofit to window',
        full: true,
      );
      notifyListeners();
      return;
    }

    await _session.applyEngineStyle(
      () async {
        if (values.clearTableBorder) {
          if (!await _host.engine!.setTableBorderAsync(
            caretRunId: caret,
            width: 0,
            color: Colors.black,
          )) {
            return false;
          }
        } else if (values.tableBorderWidth > 0) {
          if (!await _host.engine!.setTableBorderAsync(
            caretRunId: caret,
            width: values.tableBorderWidth,
            color: values.tableBorderColor,
          )) {
            return false;
          }
        }

        if (values.clearCellShading) {
          if (!await _host.engine!.setTableCellShadingAsync(caretRunId: caret)) {
            return false;
          }
        } else if (values.cellShading != null) {
          if (!await _host.engine!.setTableCellShadingAsync(
            caretRunId: caret,
            shading: values.cellShading,
          )) {
            return false;
          }
        }

        if (values.columnWidth != null && values.columnWidth! > 0) {
          return _host.engine!.resizeTableColumnAsync(
            caretRunId: caret,
            width: values.columnWidth!,
          );
        }
        return true;
      },
      'Table design applied',
      full: true,
    );
    notifyListeners();
  }

  Future<void> sortTableAscending() async {
    if (!await _prepareTableCellEdit()) return;
    await _session.applyEngineStyle(
      () => _host.engine!.sortTableRowsAsync(
        caretRunId: _selection.defaultRunId(),
        ascending: true,
      ),
      'Table sorted A→Z',
      full: true,
    );
    notifyListeners();
  }

  Future<void> sortTableDescending() async {
    if (!await _prepareTableCellEdit()) return;
    await _session.applyEngineStyle(
      () => _host.engine!.sortTableRowsAsync(
        caretRunId: _selection.defaultRunId(),
        ascending: false,
      ),
      'Table sorted Z→A',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertNestedTable() async {
    if (!await _prepareTableCellEdit()) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertNestedTableAsync(
        caretRunId: _selection.defaultRunId(),
        rows: 2,
        cols: 2,
      ),
      'Nested table inserted (2×2)',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertTableSumFormula() async {
    if (!await _prepareTableCellEdit()) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertTableSumFieldAsync(
        caretRunId: _selection.defaultRunId(),
      ),
      'SUM formula inserted',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertImageBytes(Uint8List bytes, String mimeType) async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertImageBytesAsync(
        bytes,
        mimeType,
        caretRunId: _selection.defaultRunId(),
      ),
      'Picture inserted',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertImage() async {
    if (!_host.isConnected) return;
    _session.setStatusText('Choose picture…');
    notifyListeners();
    try {
      final picked = await pickDocumentFile(
        dialogTitle: 'Insert picture',
        allowedExtensions: const ['png', 'jpg', 'jpeg', 'svg'],
      );
      if (picked == null) {
        _session.setStatusText('Insert cancelled');
        return;
      }
      final mime = _mimeForPicture(picked.name.split('.').last, picked.name);
      await insertImageBytes(picked.bytes, mime);
    } catch (e) {
      _session.setStatusText('Insert failed: $e');
    }
  }

  String _mimeForPicture(String? extension, String name) {
    final ext = (extension ?? name.split('.').last).toLowerCase();
    return switch (ext) {
      'png' => 'image/png',
      'jpg' || 'jpeg' => 'image/jpeg',
      'svg' => 'image/svg+xml',
      _ => 'application/octet-stream',
    };
  }

  void applyHeading1() => applyParagraphStyle('Heading 1');

  void applyNormalStyle() => applyParagraphStyle('Normal');

  void applyParagraphStyle(String styleName) => _session.applyEngineStyle(
        () => _host.engine!.applyParagraphStyleAsync(
          styleName: styleName,
          caretRunId: _selection.defaultRunId(),
        ),
        '$styleName applied',
      ).then((_) => _formatting.setActiveParagraphStyle(styleName));

  void applyDocumentTheme(String themeName) => _session.applyEngineStyle(
        () => _host.engine!.applyDocumentThemeAsync(themeName: themeName),
        '$themeName theme applied',
      ).then((_) {
        _documentThemeName = themeName;
        notifyListeners();
      });

  Map<String, dynamic> _currentSectionFormat() {
    final json = _host.engine?.fetchSectionFormat(
      caretRunId: _selection.defaultRunId(),
    );
    if (json != null && json.isNotEmpty) {
      try {
        return jsonDecode(json) as Map<String, dynamic>;
      } catch (_) {}
    }
    return PageSetupPresets.defaultSectionFormat();
  }

  Future<void> _applySectionFormat(Map<String, dynamic> next, String status) =>
      _session.applyEngineStyle(
        () => _host.engine!.applySectionFormatJsonAsync(
          formatJson: jsonEncode(next),
          caretRunId: _selection.defaultRunId(),
        ),
        status,
        full: true,
      ).then((_) => notifyListeners());

  void applyMarginPreset(String name) => _applySectionFormat(
        PageSetupPresets.withMarginPreset(_currentSectionFormat(), name),
        '$name margins applied',
      );

  void setOrientation({required bool landscape}) => _applySectionFormat(
        PageSetupPresets.withOrientation(
          _currentSectionFormat(),
          landscape: landscape,
        ),
        landscape ? 'Landscape orientation applied' : 'Portrait orientation applied',
      );

  void applyPageSizePreset(String name) => _applySectionFormat(
        PageSetupPresets.withPageSizePreset(_currentSectionFormat(), name),
        '$name page size applied',
      );

  void applyColumnCount(int count) => _applySectionFormat(
        PageSetupPresets.withColumnCount(_currentSectionFormat(), count),
        switch (count.clamp(1, 3)) {
          2 => 'Two columns applied',
          3 => 'Three columns applied',
          _ => 'One column applied',
        },
      );

  void applyPageColor(Color? color) => _applySectionFormat(
        PageSetupPresets.withPageColor(_currentSectionFormat(), color),
        color == null ? 'Page color cleared' : 'Page color applied',
      );

  void applyPageBorders({required double width, required Color color}) =>
      _applySectionFormat(
        PageSetupPresets.withPageBorders(
          _currentSectionFormat(),
          width: width,
          color: color,
        ),
        'Page borders applied',
      );

  void clearPageBorders() => _applySectionFormat(
        PageSetupPresets.withoutPageBorders(_currentSectionFormat()),
        'Page borders removed',
      );

  /// Design/Layout → Page Borders dialog.
  Future<void> editPageBorders(BuildContext context) async {
    final values = await PageBordersDialog.show(
      context,
      initial: PageBordersValues(
        enabled: hasPageBorders,
        width: pageBorderWidth ?? 1.0,
        color: pageBorderColor ?? Colors.black,
      ),
    );
    if (values == null) return;
    if (values.clear || !values.enabled) {
      clearPageBorders();
    } else {
      applyPageBorders(width: values.width, color: values.color);
    }
  }

  void applyWatermark(String text) => _applySectionFormat(
        PageSetupPresets.withWatermark(_currentSectionFormat(), text),
        'Watermark applied',
      );

  void clearWatermark() => _applySectionFormat(
        PageSetupPresets.withoutWatermark(_currentSectionFormat()),
        'Watermark removed',
      );

  void setLineNumbersEnabled(bool enabled) => _applySectionFormat(
        PageSetupPresets.withLineNumbers(
          _currentSectionFormat(),
          enabled: enabled,
        ),
        enabled ? 'Line numbers enabled' : 'Line numbers disabled',
      );

  void setDifferentFirstPage(bool enabled) {
    _applySectionFormat(
      PageSetupPresets.withDifferentFirstPage(_currentSectionFormat(), enabled),
      enabled ? 'Different first page on' : 'Different first page off',
    );
    if (_editZone != DocumentEditZone.body) {
      unawaited(
        _activateHeaderFooterEdit(isHeader: _editZone == DocumentEditZone.header),
      );
    }
  }

  Future<void> setEvenAndOddHeaders(bool enabled) async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.setEvenAndOddHeadersAsync(enabled: enabled),
      enabled ? 'Odd & even headers on' : 'Odd & even headers off',
      full: true,
    );
    if (_editZone != DocumentEditZone.body) {
      await _activateHeaderFooterEdit(
        isHeader: _editZone == DocumentEditZone.header,
      );
    }
    notifyListeners();
  }

  Future<void> setHeaderFooterLinked(bool linked) async {
    if (!_host.isConnected || _editZone == DocumentEditZone.body) return;
    await _session.applyEngineStyle(
      () => _host.engine!.setHeaderFooterLinkAsync(
        caretRunId: _selection.defaultRunId(),
        isHeader: _editZone == DocumentEditZone.header,
        linked: linked,
        pageIndex: _selection.caretPage,
      ),
      linked ? 'Linked to previous' : 'Unlinked from previous',
      full: true,
    );
    if (!linked) {
      await _activateHeaderFooterEdit(
        isHeader: _editZone == DocumentEditZone.header,
      );
    }
    notifyListeners();
  }

  void applyNumberedList() => _session.applyEngineStyle(
        () => _host.engine!.applyNumberedListStyleAsync(caretRunId: _selection.defaultRunId()),
        'Numbered list applied',
      );

  void applyBulletList() => _session.applyEngineStyle(
        () => _host.engine!.applyBulletListStyleAsync(caretRunId: _selection.defaultRunId()),
        'Bullet list applied',
      );

  void restartNumbering() => _session.applyEngineStyle(
        () => _host.engine!.restartNumberingAsync(caretRunId: _selection.defaultRunId()),
        'Numbering restarted',
      );

  void continueNumbering() => _session.applyEngineStyle(
        () => _host.engine!.continueNumberingAsync(caretRunId: _selection.defaultRunId()),
        'Numbering continued',
      );

  void insertPageBreak() => _session.applyEngineStyle(
        () => _host.engine!.insertPageBreakAtAsync(caretRunId: _selection.defaultRunId()),
        'Page break inserted',
      );

  /// Insert → Cover Page: title block at the document start, then a page break.
  Future<void> insertCoverPage(BuildContext context) async {
    if (!_host.isConnected || _host.engine == null) return;
    final values = await CoverPageDialog.show(
      context,
      initialTitle:
          documentTitle == 'Document1' ? 'Document Title' : documentTitle,
    );
    if (values == null) return;
    await insertCoverPageContent(
      title: values.title,
      subtitle: values.subtitle,
      author: values.author,
    );
  }

  /// Inserts cover text at offset 0, styles the title, then adds a page break.
  Future<void> insertCoverPageContent({
    required String title,
    String subtitle = '',
    String author = '',
  }) async {
    if (!_host.isConnected || _host.engine == null) return;
    final runId = _selection.defaultRunId() ??
        (_host.engine is MockDocumentEngine
            ? (_host.engine as MockDocumentEngine).defaultRunId
            : null);
    if (runId == null) {
      _session.setStatusText('Cover page failed: no caret run');
      notifyListeners();
      return;
    }

    final lines = <String>[title.trim()];
    if (subtitle.trim().isNotEmpty) lines.add(subtitle.trim());
    if (author.trim().isNotEmpty) lines.add(author.trim());
    final body = '${lines.join('\n')}\n';

    _selection.setCaret(runId, 0, page: 0);
    final ok = await applyAiTextSuggestion(
      runId: runId,
      start: 0,
      end: 0,
      text: body,
    );
    if (!ok) {
      _session.setStatusText('Cover page insert failed');
      notifyListeners();
      return;
    }

    _selection.setCaret(runId, 0, page: 0);
    await _session.applyEngineStyle(
      () => _host.engine!.applyParagraphStyleAsync(
        styleName: 'Heading 1',
        caretRunId: runId,
      ),
      'Cover title styled',
    );
    _formatting.setActiveParagraphStyle('Heading 1');

    _selection.setCaret(runId, body.length, page: 0);
    await _session.applyEngineStyle(
      () => _host.engine!.insertPageBreakAtAsync(caretRunId: runId),
      'Cover page inserted',
      full: true,
    );
    notifyListeners();
  }

  void insertSectionBreak() => _session.applyEngineStyle(
        () => _host.engine!.insertSectionBreakAtAsync(caretRunId: _selection.defaultRunId()),
        'Section break inserted',
        full: true,
      );

  Future<void> openHeaderEdit() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.ensureHeaderFooterAsync(
        isHeader: true,
        caretRunId: _selection.defaultRunId(),
        pageIndex: _selection.caretPage,
      ),
      'Editing header',
      full: true,
    );
    await _activateHeaderFooterEdit(isHeader: true);
  }

  Future<void> openFooterEdit() async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.ensureHeaderFooterAsync(
        isHeader: false,
        caretRunId: _selection.defaultRunId(),
        pageIndex: _selection.caretPage,
      ),
      'Editing footer',
      full: true,
    );
    await _activateHeaderFooterEdit(isHeader: false);
  }

  Future<void> _activateHeaderFooterEdit({required bool isHeader}) async {
    final seed = _host.engine!.fetchHeaderFooterSeedRun(
      caretRunId: _selection.defaultRunId(),
      isHeader: isHeader,
      pageIndex: _selection.caretPage,
    );
    if (seed == null) {
      _session.setStatusText(
        isHeader ? 'Header edit failed' : 'Footer edit failed',
      );
      notifyListeners();
      return;
    }
    clearDiagramSelection();
    clearImageSelection();
    _editZone = isHeader ? DocumentEditZone.header : DocumentEditZone.footer;
    _selection.setCaret(seed, 0, page: _selection.caretPage);
    _selection.syncCaretGeometry();
    _formatting.syncFromCaret();
    ensureCaretVisible(notify: false);
    requestEditorFocus();
  }

  void closeHeaderFooterEdit() {
    if (_editZone == DocumentEditZone.body) return;
    _editZone = DocumentEditZone.body;
    _selection.ensureGlyphCaret();
    _selection.syncCaretGeometry();
    notifyListeners();
  }

  Future<void> insertPageNumberField() async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();

    // Page numbers belong in a header/footer band — never insert into body text.
    if (_editZone == DocumentEditZone.body) {
      await openFooterEdit();
    }

    final runId = _selection.defaultRunId();
    if (runId == null) return;
    final offset = _runCharLength(runId);

    await _session.applyEngineStyle(
      () => _host.engine!.insertFieldAsync(
        runId: runId,
        offset: offset,
        fieldType: 'page',
      ),
      'Page number inserted',
      full: true,
    );

    _moveCaretToHeaderFooterEnd();
    requestEditorFocus();
    notifyListeners();
  }

  Future<void> insertDateField() async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();

    if (_editZone == DocumentEditZone.body) {
      await openFooterEdit();
    }

    final runId = _selection.defaultRunId();
    if (runId == null) return;
    final offset = _runCharLength(runId);

    await _session.applyEngineStyle(
      () => _host.engine!.insertFieldAsync(
        runId: runId,
        offset: offset,
        fieldType: 'date',
      ),
      'Date inserted',
      full: true,
    );

    _moveCaretToHeaderFooterEnd();
    notifyListeners();
  }

  int _runCharLength(String runId) {
    final engine = _host.engine;
    if (engine == null) return 0;
    var len = 0;
    while (true) {
      final ch = engine.fetchTextRange(runId, len, runId, len + 1);
      if (ch == null || ch.isEmpty) break;
      len++;
    }
    return len;
  }

  void _moveCaretToHeaderFooterEnd() {
    final page = _selection.caretPage;
    final x = pageWidth - marginRight - 8;
    final y = _editZone == DocumentEditZone.footer
        ? pageHeight - (marginBottom * 0.5)
        : marginTop * 0.5;
    _selection.hitTestAt(page, x, y);
    _selection.syncCaretGeometry();
  }

  /// Inserts a footnote reference at the caret (F16.S1).
  Future<void> insertFootnote(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertFootnoteAsync(
        runId: runId,
        offset: _selection.caretOffset,
      ),
      'Footnote inserted',
      full: true,
    );
    requestEditorFocus();
  }

  /// Inserts an endnote reference at the caret.
  Future<void> insertEndnote(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertEndnoteAsync(
        runId: runId,
        offset: _selection.caretOffset,
      ),
      'Endnote inserted',
      full: true,
    );
    requestEditorFocus();
  }

  /// Inserts a comment anchor at the caret (F17.S3).
  Future<void> insertComment(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    final body = await CommentDialog.show(context);
    if (body == null) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertCommentAsync(
        runId: runId,
        offset: _selection.caretOffset,
        bodyText: body,
      ),
      'Comment inserted',
      full: true,
    );
    notifyListeners();
  }

  /// Materializes a table of contents from heading styles (F16.S2).
  Future<void> insertTableOfContents(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    await _session.applyEngineStyle(
      () => _host.engine!.insertTableOfContentsAsync(caretRunId: runId),
      'Table of contents inserted',
      full: true,
    );
    requestEditorFocus();
  }

  /// Materializes a table of figures from caption paragraphs.
  Future<void> insertTableOfFigures(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    await _session.applyEngineStyle(
      () => _host.engine!.insertTableOfFiguresAsync(caretRunId: runId),
      'Table of figures inserted',
      full: true,
    );
    requestEditorFocus();
  }

  /// Inserts a citation for the default sample source (F16.S3).
  Future<void> insertCitation(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    const key = 'Smith2020';
    await _session.applyEngineStyle(
      () async {
        await _host.engine!.addBibliographySourceAsync(
          key: key,
          author: 'Smith, John',
          title: 'Example Research',
          year: '2020',
        );
        return _host.engine!.insertCitationAsync(
          runId: runId,
          offset: _selection.caretOffset,
          sourceKey: key,
        );
      },
      'Citation inserted',
      full: true,
    );
    requestEditorFocus();
  }

  /// Materializes a bibliography section from cited sources (F16.S3).
  Future<void> insertBibliography(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    // Ensure at least one source exists — Bibliography alone used to fail with
    // a cryptic "Edit failed" when the document had no citations yet.
    await _session.applyEngineStyle(
      () async {
        await _host.engine!.addBibliographySourceAsync(
          key: 'Smith2020',
          author: 'Smith, John',
          title: 'Example Research',
          year: '2020',
        );
        return _host.engine!.insertBibliographyAsync(caretRunId: runId);
      },
      'Bibliography inserted',
      full: true,
    );
    requestEditorFocus();
  }

  /// Inserts a bookmark anchor at the caret (F16.S4 / F19.S3).
  Future<void> insertBookmark(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    final name = await BookmarkNameDialog.show(context);
    if (name == null || name.isEmpty) return;
    if (!context.mounted) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertBookmarkAsync(
        runId: runId,
        offset: _selection.caretOffset,
        name: name,
      ),
      'Bookmark inserted',
      full: true,
    );
    requestEditorFocus();
  }

  /// Inserts or edits a hyperlink at the caret (F19.S3).
  Future<void> insertHyperlink(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    final selected = selectedText.trim();
    final result = await HyperlinkDialog.show(
      context,
      initialText: selected,
      initialUrl: selected.startsWith('http') ? selected : 'https://',
    );
    if (result == null) return;
    if (!context.mounted) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertHyperlinkAsync(
        runId: runId,
        offset: _selection.caretOffset,
        url: result.url,
        text: result.text,
        tooltip: result.tooltip,
      ),
      'Hyperlink inserted',
      full: true,
    );
    notifyListeners();
  }

  /// Inserts a plain-text or checkbox form field at the caret (F26.S1).
  Future<void> insertFormField(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    final result = await FormFieldDialog.show(context);
    if (result == null) return;
    if (!context.mounted) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertFormFieldAsync(
        runId: runId,
        offset: _selection.caretOffset,
        kind: result.kindWire,
        name: result.name,
        initialValue: result.initialValueWire,
      ),
      'Form field inserted',
      full: true,
    );
    notifyListeners();
  }

  /// Updates / toggles an existing form field value (F26.S1).
  Future<void> setFormFieldValue({
    required String runId,
    required String value,
  }) async {
    if (!_host.isConnected) return;
    await _session.applyEngineStyle(
      () => _host.engine!.setFormFieldValueAsync(runId: runId, value: value),
      'Form field updated',
      full: true,
    );
    notifyListeners();
  }

  /// Active mail-merge CSV data source (F26.S2).
  MailMergeDataSource? get mailMergeData => _mailMergeData;

  /// Whether a mail-merge data source is loaded (F26.S2).
  bool get hasMailMergeData =>
      _mailMergeData != null && _mailMergeData!.rows.isNotEmpty;

  /// Opens Start Mail Merge and loads a CSV data source (F26.S2).
  Future<void> startMailMerge(BuildContext context) async {
    final data = await StartMailMergeDialog.show(context);
    if (data == null) return;
    _mailMergeData = data;
    _mailMergeRowIndex = 0;
    _session.setStatusText(
      'Mail merge: ${data.rows.length} row(s), ${data.headers.length} field(s)',
    );
    notifyListeners();
  }

  /// Inserts a merge field at the caret (F26.S2).
  Future<void> insertMergeField(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    final name = await InsertMergeFieldDialog.show(
      context,
      headers: _mailMergeData?.headers ?? const [],
    );
    if (name == null || name.trim().isEmpty) return;
    if (!context.mounted) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertMergeFieldAsync(
        runId: runId,
        offset: _selection.caretOffset,
        name: name.trim(),
      ),
      'Merge field inserted',
      full: true,
    );
    notifyListeners();
  }

  /// In-app plugin registry (capability-gated host mirror of F26.S3).
  PluginRegistry get pluginRegistry => _pluginRegistry;

  /// Flutter AI facade (production providers + hybrid routing; F28.S1).
  AiClient get aiClient => _aiClient;

  /// Opens AI routing / provider settings (F28.S1).
  Future<void> openAiSettings(BuildContext context) async {
    await AiSettingsDialog.show(
      context,
      client: _aiClient,
      onChanged: notifyListeners,
    );
    notifyListeners();
  }

  /// Opens document chat with RAG citations (F28.S3).
  Future<void> openDocumentChat(BuildContext context) async {
    await _host.ensureLayoutReady();
    if (!context.mounted) return;
    await AiChatDialog.show(
      context,
      client: _aiClient,
      documentText: documentText,
      onCitationTap: (paragraphId) {
        _session.setStatusText('Citation: $paragraphId');
        notifyListeners();
      },
    );
    notifyListeners();
  }

  /// Generate outline/minutes/report into a new document (F28.S4).
  Future<void> openContentGenerate(BuildContext context) async {
    final generated = await AiGenerateDialog.show(
      context,
      client: _aiClient,
    );
    if (generated == null) return;
    await openGeneratedDocument(generated);
  }

  /// Smart edit: suggest headings + TOC draft; apply only on Accept (F28.S6).
  Future<void> openSmartEdit(BuildContext context) async {
    await _host.ensureLayoutReady();
    if (!context.mounted) return;
    final plan = await AiSmartEditDialog.show(
      context,
      client: _aiClient,
      documentText: documentText,
    );
    if (plan == null) return;
    await applySmartEditPlan(plan);
  }

  final List<String> _smartEditAppliedStyles = [];

  /// Styles applied by the last smart-edit accept (tests / status).
  List<String> get smartEditAppliedStyles =>
      List.unmodifiable(_smartEditAppliedStyles);

  /// Apply an accepted smart-edit plan (headings then optional TOC).
  Future<void> applySmartEditPlan(AiSmartEditPlan plan) async {
    if (!_host.isConnected || _host.engine == null) return;
    _smartEditAppliedStyles.clear();
    for (final heading in plan.headings) {
      final matches = _host.engine!.findMatches(heading.previewText, false);
      final runId = matches != null && matches.isNotEmpty
          ? matches.first.startRunId
          : _selection.defaultRunId();
      await _session.applyEngineStyle(
        () => _host.engine!.applyParagraphStyleAsync(
          styleName: heading.styleName,
          caretRunId: runId,
        ),
        '${heading.styleName} applied',
      );
      _formatting.setActiveParagraphStyle(heading.styleName);
      _smartEditAppliedStyles.add(heading.styleName);
    }
    if (plan.insertToc) {
      final engine = _host.engine!;
      if (engine is MockDocumentEngine) {
        engine.setMockOutlineHeadingsForTest([
          for (final h in plan.headings) (text: h.previewText, page: 1),
        ]);
      }
      final runId = _selection.defaultRunId();
      await _session.applyEngineStyle(
        () => engine.insertTableOfContentsAsync(caretRunId: runId),
        'Table of contents inserted',
        full: true,
      );
    }
    final notes = plan.autoFormatNotes.isEmpty
        ? ''
        : ' · ${plan.autoFormatNotes.length} format note(s)';
    _session.setStatusText(
      'Smart edit applied: ${_smartEditAppliedStyles.length} heading(s)'
      '${plan.insertToc ? ' + TOC' : ''}$notes',
    );
    notifyListeners();
  }

  /// Suggest a table / diagram / timeline and insert on Accept (F28.S5).
  Future<void> openVisualAssist(BuildContext context) async {
    final topic = selectedText.trim().isNotEmpty
        ? selectedText.trim()
        : documentText.trim().isNotEmpty
            ? documentText.trim().split('\n').first
            : 'document overview';
    final suggestion = await AiVisualDialog.show(
      context,
      client: _aiClient,
      initialTopic: topic,
    );
    if (suggestion == null) return;
    await applyVisualSuggestion(suggestion);
  }

  /// Insert the suggested visual as a document block (table / SmartArt / timeline table).
  Future<void> applyVisualSuggestion(AiVisualSuggestion suggestion) async {
    if (!_host.isConnected) return;
    switch (suggestion.kind.type) {
      case AiVisualKindType.table:
        await insertTable(
          rows: suggestion.kind.rows,
          cols: suggestion.kind.cols,
        );
        _session.setStatusText(
          'Inserted table: ${suggestion.title} '
          '(${suggestion.kind.rows}×${suggestion.kind.cols})',
        );
      case AiVisualKindType.diagram:
        await insertSmartArt(diagramType: suggestion.kind.diagramType);
        _session.setStatusText('Inserted diagram: ${suggestion.title}');
      case AiVisualKindType.timeline:
        final cols = suggestion.kind.stages.length.clamp(2, 8);
        await insertTable(rows: 1, cols: cols);
        _session.setStatusText(
          'Inserted timeline: ${suggestion.title} '
          '(${suggestion.kind.stages.join(' → ')})',
        );
    }
    notifyListeners();
  }

  /// Replace the session with [generated] content (new untitled document).
  Future<void> openGeneratedDocument(AiGeneratedDocument generated) async {
    if (!_host.isConnected || _host.engine == null) return;
    await newDocument();
    final text = generated.plainText;
    if (text.isEmpty) {
      _session.setStatusText('Generated empty ${generated.kind.label}');
      notifyListeners();
      return;
    }
    final runId = _selection.defaultRunId() ??
        (_host.engine is MockDocumentEngine
            ? (_host.engine as MockDocumentEngine).defaultRunId
            : null);
    if (runId == null) {
      _session.setStatusText('Generate failed: no caret run');
      notifyListeners();
      return;
    }
    final ok = await applyAiTextSuggestion(
      runId: runId,
      start: 0,
      end: 0,
      text: text,
    );
    _session.setStatusText(
      ok
          ? 'New ${generated.kind.label.toLowerCase()}: ${generated.topic}'
          : 'Generate apply failed',
    );
    notifyListeners();
  }

  String get proofingLanguageId => _proofingLanguageId;

  ProofingLanguage get proofingLanguage =>
      proofingLanguageById(_proofingLanguageId);

  void setProofingLanguage(String languageId) {
    _proofingLanguageId = proofingLanguageById(languageId).id;
    _session.setStatusText('Proofing language: ${proofingLanguage.label}');
    notifyListeners();
  }

  /// Review → Language: choose proofing / default translate language.
  Future<void> chooseProofingLanguage(BuildContext context) async {
    final selected = await LanguageDialog.show(
      context,
      currentLanguageId: _proofingLanguageId,
    );
    if (selected == null) return;
    setProofingLanguage(selected);
  }

  /// Review → Translate: AI-translate the selection into a chosen language.
  Future<void> translateSelection(BuildContext context) async {
    if (!_host.isConnected || _host.engine == null) return;
    if (!_selection.selectWordAtCaret()) {
      _session.setStatusText('Select text to translate');
      notifyListeners();
      if (context.mounted) {
        ScaffoldMessenger.maybeOf(context)?.showSnackBar(
          const SnackBar(content: Text('Select text to translate')),
        );
      }
      return;
    }
    final original = selectedText;
    if (original.isEmpty) {
      _session.setStatusText('Select text to translate');
      notifyListeners();
      return;
    }
    final range = _selection.selection;
    if (range == null) return;
    final (start, end) = range.normalized();
    if (start.runId != end.runId) {
      _session.setStatusText('Translate supports a single-run selection');
      notifyListeners();
      return;
    }

    if (!context.mounted) return;
    final targetId = await AiTranslateDialog.pickLanguage(
      context,
      initialLanguageId: _proofingLanguageId == 'en-US' ? 'es' : _proofingLanguageId,
    );
    if (targetId == null) {
      _session.setStatusText('Translate cancelled');
      notifyListeners();
      return;
    }
    final target = proofingLanguageById(targetId);

    _session.setStatusText('Translating…');
    notifyListeners();
    late final String suggestion;
    try {
      suggestion = await _aiClient.translate(
        AiDocumentContext(
          selectionText: original,
          pageCount: 1,
          totalTokenEstimate: original.length,
        ),
        target.translateName,
      );
    } catch (e) {
      _session.setStatusText('Translate failed: $e');
      notifyListeners();
      return;
    }

    if (!context.mounted) return;
    final accepted = await AiTranslateDialog.showResult(
      context,
      original: original,
      suggestion: suggestion,
      targetLanguageId: target.id,
    );
    if (accepted == null) {
      _session.setStatusText('Translate discarded');
      notifyListeners();
      return;
    }

    final lo = start.offset < end.offset ? start.offset : end.offset;
    final hi = start.offset < end.offset ? end.offset : start.offset;
    final ok = await applyAiTextSuggestion(
      runId: start.runId,
      start: lo,
      end: hi,
      text: accepted,
    );
    _session.setStatusText(ok ? 'Translation applied' : 'Translate apply failed');
    notifyListeners();
  }

  /// Review → Thesaurus: suggest synonyms and optionally replace the word.
  Future<void> openThesaurus(BuildContext context) async {
    if (!_host.isConnected || _host.engine == null) return;
    if (!_selection.selectWordAtCaret()) {
      _session.setStatusText('Place the caret in a word for thesaurus');
      notifyListeners();
      if (context.mounted) {
        ScaffoldMessenger.maybeOf(context)?.showSnackBar(
          const SnackBar(content: Text('Place the caret in a word for thesaurus')),
        );
      }
      return;
    }
    final original = selectedText;
    final lemma = thesaurusLemma(original);
    if (lemma.isEmpty) {
      _session.setStatusText('Place the caret in a word for thesaurus');
      notifyListeners();
      return;
    }
    final range = _selection.selection;
    if (range == null) return;
    final (start, end) = range.normalized();
    if (start.runId != end.runId) {
      _session.setStatusText('Thesaurus supports a single-run selection');
      notifyListeners();
      return;
    }

    final synonyms = <String>{...lookupThesaurus(lemma)};
    try {
      final aiRaw = await _aiClient.suggestSynonyms(
        AiDocumentContext(
          selectionText: lemma,
          pageCount: 1,
          totalTokenEstimate: lemma.length,
        ),
      );
      synonyms.addAll(parseThesaurusAiResponse(aiRaw, exclude: lemma));
    } catch (_) {
      // Offline / unconfigured AI — local dictionary is enough.
    }

    if (!context.mounted) return;
    final chosen = await ThesaurusDialog.show(
      context,
      word: lemma,
      synonyms: synonyms.toList(growable: false),
    );
    if (chosen == null) {
      _session.setStatusText(
        synonyms.isEmpty ? 'No thesaurus matches' : 'Thesaurus cancelled',
      );
      notifyListeners();
      return;
    }

    // Preserve simple capitalization of the original selection.
    var replacement = chosen;
    final trimmed = original.trim();
    if (trimmed.isNotEmpty &&
        trimmed[0].toUpperCase() == trimmed[0] &&
        trimmed.toLowerCase() != trimmed) {
      replacement = chosen[0].toUpperCase() + chosen.substring(1);
    }

    final lo = start.offset < end.offset ? start.offset : end.offset;
    final hi = start.offset < end.offset ? end.offset : start.offset;
    final ok = await applyAiTextSuggestion(
      runId: start.runId,
      start: lo,
      end: hi,
      text: replacement,
    );
    _session.setStatusText(ok ? 'Replaced with “$replacement”' : 'Replace failed');
    notifyListeners();
  }

  /// Rewrite the current selection via AI and apply on Accept (F28.S2).
  Future<void> rewriteForAudience(BuildContext context) async {
    final tone = await AudienceRewriteDialog.pick(context);
    if (tone == null || !context.mounted) return;
    await rewriteSelection(context, tone: tone);
  }

  Future<void> openConsistencyChecker(BuildContext context) async {
    if (!_host.isConnected || _host.engine == null) return;
    final engine = _host.engine!;
    final text = engine is MockDocumentEngine
        ? engine.text
        : (selectedText.isNotEmpty ? selectedText : documentText);
    final findings = await scanWithAi(_aiClient, text);
    if (!context.mounted) return;
    await ConsistencyCheckerDialog.show(
      context: context,
      findings: findings,
      onApply: (finding) async {
        if (finding.suggestion == null) return;
        var total = 0;
        for (final variant in finding.variants) {
          if (variant == finding.suggestion) continue;
          final count = await engine.replaceAll(
            variant,
            finding.suggestion!,
            true,
          );
          if (count != null) total += count;
        }
        _session.setStatusText(
          total > 0
              ? 'Applied ${finding.suggestion} ($total replacement(s))'
              : 'Applied ${finding.suggestion}',
        );
        _session.markDocumentDirty();
        notifyListeners();
      },
    );
  }

  Future<void> createEnvelope(BuildContext context) async {
    final result = await EnvelopesLabelsDialog.show(
      context,
      kind: MailCreateKind.envelope,
    );
    if (result == null) return;
    await newDocument();
    final format = Map<String, dynamic>.from(_currentSectionFormat())
      ..['page_width'] = result.pageWidth
      ..['page_height'] = result.pageHeight;
    await _applySectionFormat(format, 'Envelope created');
    for (final ch in result.address.characters) {
      if (ch == '\n') {
        await insertGlyphParagraphBreak();
      } else {
        await insertGlyphCharacter(ch);
      }
    }
    _session.setStatusText('Envelope created');
    notifyListeners();
  }

  Future<void> createLabel(BuildContext context) async {
    final result = await EnvelopesLabelsDialog.show(
      context,
      kind: MailCreateKind.label,
    );
    if (result == null) return;
    await newDocument();
    final format = Map<String, dynamic>.from(_currentSectionFormat())
      ..['page_width'] = result.pageWidth
      ..['page_height'] = result.pageHeight;
    await _applySectionFormat(format, 'Label created');
    for (final ch in result.address.characters) {
      if (ch == '\n') {
        await insertGlyphParagraphBreak();
      } else {
        await insertGlyphCharacter(ch);
      }
    }
    _session.setStatusText('Label created');
    notifyListeners();
  }

  Future<void> insertNextRecordField(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertFieldAsync(
        runId: runId,
        offset: _selection.caretOffset,
        fieldType: 'next',
      ),
      'Next Record field inserted',
      full: true,
    );
    notifyListeners();
  }

  Future<void> insertMergeIfField(BuildContext context) async {
    final field = await showDialog<String>(
      context: context,
      builder: (context) {
        final controller = TextEditingController(text: 'City');
        return AlertDialog(
          title: const Text('Insert IF field'),
          content: TextField(
            key: const Key('merge_if_field_name'),
            controller: controller,
            decoration: const InputDecoration(labelText: 'Merge field name'),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Cancel'),
            ),
            FilledButton(
              key: const Key('merge_if_insert'),
              onPressed: () => Navigator.of(context).pop(controller.text.trim()),
              child: const Text('Insert'),
            ),
          ],
        );
      },
    );
    if (field == null || field.isEmpty || !_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;
    await _session.applyEngineStyle(
      () => _host.engine!.insertFieldAsync(
        runId: runId,
        offset: _selection.caretOffset,
        fieldType: 'if',
        mergeName: field,
      ),
      'IF field inserted for $field',
      full: true,
    );
    notifyListeners();
  }

  /// Rewrite the current selection via AI and apply on Accept (F28.S2).
  Future<void> rewriteSelection(
    BuildContext context, {
    AiRewriteTone tone = AiRewriteTone.formal,
  }) async {
    if (!_host.isConnected || _host.engine == null) return;
    if (!_selection.hasGlyphSelection) {
      _session.setStatusText('Select text to rewrite');
      notifyListeners();
      return;
    }
    final original = selectedText;
    if (original.isEmpty) {
      _session.setStatusText('Select text to rewrite');
      notifyListeners();
      return;
    }
    final range = _selection.selection;
    if (range == null) return;
    final (start, end) = range.normalized();
    if (start.runId != end.runId) {
      _session.setStatusText('Rewrite supports a single-run selection');
      notifyListeners();
      return;
    }

    _session.setStatusText('Rewriting…');
    notifyListeners();
    late final String suggestion;
    try {
      suggestion = await _aiClient.rewrite(
        AiDocumentContext(
          selectionText: original,
          pageCount: 1,
          totalTokenEstimate: original.length,
        ),
        tone,
      );
    } catch (e) {
      _session.setStatusText('Rewrite failed: $e');
      notifyListeners();
      return;
    }

    if (!context.mounted) return;
    final accepted = await AiRewriteDialog.show(
      context,
      original: original,
      suggestion: suggestion,
      tone: tone,
    );
    if (accepted == null) {
      _session.setStatusText('Rewrite discarded');
      notifyListeners();
      return;
    }

    final lo = start.offset < end.offset ? start.offset : end.offset;
    final hi = start.offset < end.offset ? end.offset : start.offset;
    final ok = await applyAiTextSuggestion(
      runId: start.runId,
      start: lo,
      end: hi,
      text: accepted,
    );
    _session.setStatusText(ok ? 'Rewrite applied' : 'Rewrite apply failed');
    notifyListeners();
  }

  /// Apply an AI suggestion as a single-run replace (DeleteRange + InsertText).
  Future<bool> applyAiTextSuggestion({
    required String runId,
    required int start,
    required int end,
    required String text,
  }) async {
    if (_host.engine == null) return false;
    final edit = _host.performNativeEdit(
      () => _host.engine!.replaceRangeAsync(runId, start, end, text),
      dirtyPage: _selection.caretPage,
      full: true,
    );
    if (!await edit) return false;
    await _host.ensureLayoutReady();
    _selection.setCaret(runId, start + text.length, page: _selection.caretPage);
    _selection.syncCaretGeometry();
    _session.markDocumentDirty();
    notifyListeners();
    return true;
  }

  /// Opens the Plugins manager dialog (F26.S3).
  Future<void> managePlugins(BuildContext context) async {
    _syncPluginsFromHost();
    await PluginsDialog.show(
      context,
      registry: _pluginRegistry,
      onChanged: notifyListeners,
      onInstallSample: (grantEdit) => installSamplePlugin(grantEdit: grantEdit),
      onInvoke: (id) => _pluginRegistry.invoke(id),
    );
    notifyListeners();
  }

  void _syncPluginsFromHost() {
    final json = _host.engine?.fetchPluginList();
    _pluginRegistry.syncFromEngine(json);
  }

  /// Installs the sample edit plugin with optional document.edit grant (F26.S3).
  void installSamplePlugin({bool grantEdit = true}) {
    final native = _host.engine?.installSamplePluginNative(grantEdit: grantEdit);
    if (native == true) {
      _syncPluginsFromHost();
    } else {
      _pluginRegistry.installSampleEditPlugin(grantEdit: grantEdit);
    }
    _session.setStatusText(
      grantEdit
          ? 'Sample plugin installed (edit granted)'
          : 'Sample plugin installed (read-only)',
    );
    notifyListeners();
  }

  /// Invokes an installed plugin; may insert text when document.edit is granted.
  Future<void> invokePlugin(String pluginId) async {
    if (!_host.isConnected) return;
    final result = _pluginRegistry.invoke(
      pluginId,
      documentText: '',
    );
    if (result.startsWith('ERR:')) {
      _session.setStatusText(result);
      notifyListeners();
      return;
    }
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) {
      _session.setStatusText('Plugin ran but no caret run available');
      notifyListeners();
      return;
    }
    await _session.applyEngineStyle(
      () => _host.engine!.tryInsertTextAsync(
        runId,
        _selection.caretOffset,
        result,
      ),
      'Plugin inserted text',
      full: true,
    );
    notifyListeners();
  }

  /// Applies the current (or [rowIndex]) mail-merge row to the document (F26.S2).
  Future<void> finishMailMerge({int? rowIndex}) async {
    if (!_host.isConnected) return;
    final data = _mailMergeData;
    if (data == null || data.rows.isEmpty) {
      _session.setStatusText('Load a CSV via Start Mail Merge first');
      notifyListeners();
      return;
    }
    final index = rowIndex ?? _mailMergeRowIndex;
    final row = data.rowAt(index);
    if (row == null) {
      _session.setStatusText('Mail merge row out of range');
      notifyListeners();
      return;
    }
    await _session.applyEngineStyle(
      () => _host.engine!.applyMailMergeRowAsync(values: row),
      'Mail merge applied (row ${index + 1}/${data.rows.length})',
      full: true,
    );
    notifyListeners();
  }

  /// Inserts a REF field pointing at a document bookmark (F16.S4).
  Future<void> insertCrossReference(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (runId == null) return;

    var bookmarks = documentBookmarks;
    if (bookmarks.isEmpty) {
      // Hard-coded "SectionRef" used to fail silently when no bookmark existed.
      final name = await BookmarkNameDialog.show(context);
      if (name == null || name.isEmpty || !context.mounted) return;
      await _session.applyEngineStyle(
        () => _host.engine!.insertBookmarkAsync(
          runId: runId,
          offset: _selection.caretOffset,
          name: name,
        ),
        'Bookmark inserted',
        full: true,
      );
      bookmarks = documentBookmarks;
      if (bookmarks.isEmpty) {
        _session.setStatusText('Bookmark required for cross-reference');
        notifyListeners();
        return;
      }
    }

    String? bookmarkName;
    if (bookmarks.length == 1) {
      bookmarkName = bookmarks.first.name;
    } else {
      final preferred = bookmarks.where((b) => b.name == 'SectionRef');
      if (preferred.isNotEmpty) {
        bookmarkName = preferred.first.name;
      } else if (context.mounted) {
        bookmarkName = await CrossReferenceDialog.show(context, bookmarks);
      }
    }
    if (bookmarkName == null || bookmarkName.isEmpty) return;
    if (!context.mounted) return;

    await _session.applyEngineStyle(
      () => _host.engine!.insertCrossReferenceAsync(
        runId: runId,
        offset: _selection.caretOffset,
        bookmarkName: bookmarkName!,
      ),
      'Cross-reference inserted',
      full: true,
    );
    requestEditorFocus();
  }

  /// Materializes an index from bookmark targets (F16.S4).
  Future<void> insertIndex(BuildContext context) async {
    if (!_host.isConnected) return;
    _selection.ensureGlyphCaret();
    final runId = _selection.defaultRunId();
    if (documentBookmarks.isEmpty) {
      _session.setStatusText('Insert bookmarks first, then Insert Index');
      notifyListeners();
      return;
    }
    await _session.applyEngineStyle(
      () => _host.engine!.insertIndexAsync(caretRunId: runId),
      'Index inserted',
      full: true,
    );
    requestEditorFocus();
  }

  // ── Legacy stubs (removed TextField path) ─────────────────────────────────
  @Deprecated('TextField fallback removed in R2.4')
  void attachTextEditor(TextEditingController c, FocusNode f) {}

  @Deprecated('TextField fallback removed in R2.4')
  void detachTextEditor(TextEditingController c) {}

  @Deprecated('TextField fallback removed in R2.4')
  TextEditingController? get textController => null;

  @Deprecated('TextField fallback removed in R2.4')
  String textForPage(int pageIndex) => _host.documentText;

  @Deprecated('TextField fallback removed in R2.4')
  void replacePageText(int pageIndex, String text) {
    _host.setDocumentText(text);
    notifyListeners();
  }

  // ── Test hooks ────────────────────────────────────────────────────────────
  @visibleForTesting
  void setDocumentReadOnlyForTest(bool readOnly) =>
      _session.setDocumentReadOnlyForTest(readOnly);

  @visibleForTesting
  void setDisplayListForTest(Uint8List bytes, {bool preferTextRendering = false, int pageCount = 1}) {
    _host.injectDisplayListForTest(bytes, pageCount: pageCount);
    notifyListeners();
  }

  // ── Private helpers ───────────────────────────────────────────────────────
  Future<void> _completeGlyphDrag(int pageIndex, double x, double y) async {
    _selection.completeGlyphDrag(pageIndex, x, y);
    await moveGlyphSelectionTo(pageIndex, x, y);
  }

  Future<bool> _tryPasteFormatted(String runId, EditorClipboardPayload payload) async {
    if (payload.docxBytes != null && payload.docxBytes!.isNotEmpty) {
      final edit = _host.performNativeEdit(
        () => _host.engine!.tryPasteDocxAsync(runId, _selection.caretOffset, payload.docxBytes!),
        full: true,
      );
      if (await edit) {
        await _host.ensureLayoutReady();
        _selection.syncCaretGeometry();
        return true;
      }
    }
    final html = payload.html?.trim();
    if (html != null && html.isNotEmpty) {
      final edit = _host.performNativeEdit(
        () => _host.engine!.tryPasteHtmlAsync(runId, _selection.caretOffset, html),
        full: true,
      );
      if (await edit) {
        await _host.ensureLayoutReady();
        _selection.syncCaretGeometry();
        return true;
      }
    }
    return false;
  }

  Future<bool> _tryPastePlain(String runId, String? text) async {
    if (text == null || text.isEmpty) return false;
    final normalized = _normalizeClipboardText(text);
    if (normalized.isEmpty) return false;
    final wantsFull = normalized.contains('\n') || normalized.contains('\r');
    final edit = _host.performNativeEdit(
      () => _host.engine!.tryInsertTextAsync(runId, _selection.caretOffset, normalized),
      dirtyPage: wantsFull ? null : _selection.caretPage,
      full: wantsFull,
    );
    if (!await edit) return false;
    // Multi-paragraph paste moves the caret onto a new run; sync from geometry.
    if (wantsFull) {
      await _host.ensureLayoutReady();
      _selection.syncCaretGeometry();
    } else {
      _selection.afterInsert(runId, _selection.caretOffset + normalized.length);
    }
    return true;
  }

  /// Map Word Wingdings/PUA and strip controls that paint as tofu boxes.
  static String _normalizeClipboardText(String text) {
    final normalizedBreaks = text.replaceAll('\r\n', '\n').replaceAll('\r', '\n');
    final out = StringBuffer();
    for (final ch in normalizedBreaks.runes) {
      if (ch == 0x0A || ch == 0x09) {
        out.writeCharCode(ch);
        continue;
      }
      if (ch == 0xA0) {
        out.writeCharCode(0x20);
        continue;
      }
      if (ch == 0x200B || ch == 0x200C || ch == 0x200D || ch == 0xFEFF || ch == 0xAD) {
        continue;
      }
      if (ch >= 0xF0E0 && ch <= 0xF0EF) {
        out.write('→');
        continue;
      }
      if (ch == 0xF035 || ch == 0xF0B6 || ch == 0xF0B7 || ch == 0xF0A7 || ch == 0xF0A8) {
        out.write('•');
        continue;
      }
      if (ch >= 0xF000 && ch <= 0xF8FF) {
        out.write('·');
        continue;
      }
      if (ch < 0x20 || (ch >= 0x7F && ch <= 0x9F)) {
        continue;
      }
      out.writeCharCode(ch);
    }
    return out.toString();
  }

  @override
  void dispose() {
    _disposed = true;
    HardwareKeyboard.instance.removeHandler(_onDesktopDocumentKey);
    if (identical(NativeEventRouter.instance.onUnsolicitedEvent, _onUnsolicitedEngineEvent)) {
      NativeEventRouter.instance.onUnsolicitedEvent = null;
    }
    unawaited(_tts.stop());
    _session.disposeSession();
    if (kIsWeb) {
      webGlyphFocusNode.dispose();
    }
    for (final sub in _subControllers) {
      sub.removeListener(notifyListeners);
    }
    super.dispose();
  }
}
