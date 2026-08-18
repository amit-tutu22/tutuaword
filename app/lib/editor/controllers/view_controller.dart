import 'package:flutter/foundation.dart';
import 'package:tutuaword/editor/document_view_layout.dart';

/// Zoom, page navigation, ruler, navigation pane, print preview, view layout.
class ViewController extends ChangeNotifier {
  double _zoom = 1.0;
  int _currentPage = 0;
  bool _printPreview = false;
  DocumentViewLayout _layout = DocumentViewLayout.printLayout;
  int _pageColumns = 1;
  bool _splitView = false;
  double _viewportWidth = 900;
  double _viewportHeight = 700;
  bool _showRuler = false;
  bool _showNavigationPane = false;
  bool _showStyleInspector = false;
  bool _showAccessibilityChecker = false;
  bool _showFormattingMarks = false;
  bool _preferOutlineTab = false;
  String _statusSuffix = '';
  int? _scrollRequestPage;
  CaretScrollRequest? _caretScrollRequest;

  double get zoom => _zoom;
  int get currentPage => _currentPage;
  bool get printPreview => _printPreview;
  DocumentViewLayout get layout => _layout;
  bool get isReadMode => _layout == DocumentViewLayout.readMode;
  bool get isWebLayout => _layout == DocumentViewLayout.webLayout;
  bool get isPrintLayout => _layout == DocumentViewLayout.printLayout;
  int get pageColumns => _pageColumns;
  bool get splitView => _splitView;
  double get viewportWidth => _viewportWidth;
  double get viewportHeight => _viewportHeight;
  bool get showRuler => _showRuler;
  bool get showNavigationPane => _showNavigationPane;
  bool get showStyleInspector => _showStyleInspector;
  bool get showAccessibilityChecker => _showAccessibilityChecker;
  bool get showFormattingMarks => _showFormattingMarks;
  String get statusSuffix => _statusSuffix;
  int? get scrollRequestPage => _scrollRequestPage;
  CaretScrollRequest? get caretScrollRequest => _caretScrollRequest;

  void setStatusSuffix(String value) {
    _statusSuffix = value;
  }

  void setCurrentPage(int page, int pageCount) {
    final clamped = page.clamp(0, pageCount - 1);
    if (clamped == _currentPage) return;
    _currentPage = clamped;
    notifyListeners();
  }

  void setVisiblePage(int page, int pageCount) {
    final clamped = page.clamp(0, pageCount - 1);
    if (clamped == _currentPage) return;
    _currentPage = clamped;
    notifyListeners();
  }

  void reportViewport({required double width, required double height}) {
    if (width <= 0 || height <= 0) return;
    if ((width - _viewportWidth).abs() < 0.5 &&
        (height - _viewportHeight).abs() < 0.5) {
      return;
    }
    _viewportWidth = width;
    _viewportHeight = height;
  }

  void setPrintLayout() {
    _layout = DocumentViewLayout.printLayout;
    _printPreview = false;
    notifyListeners();
  }

  void setReadMode() {
    _layout = DocumentViewLayout.readMode;
    _printPreview = false;
    notifyListeners();
  }

  void setWebLayout() {
    _layout = DocumentViewLayout.webLayout;
    _printPreview = false;
    notifyListeners();
  }

  void setPrintPreviewMode() {
    _layout = DocumentViewLayout.printLayout;
    _printPreview = true;
    notifyListeners();
  }

  void togglePrintPreview() {
    if (_printPreview) {
      setPrintLayout();
    } else {
      setPrintPreviewMode();
    }
  }

  void setZoom(double value) {
    _zoom = value.clamp(0.5, 3.0);
    notifyListeners();
  }

  void zoomIn() => setZoom(_zoom + 0.1);
  void zoomOut() => setZoom(_zoom - 0.1);

  void setPageColumns(int columns) {
    _pageColumns = columns.clamp(1, 3);
    notifyListeners();
  }

  /// Fit a single page in the current viewport.
  void zoomToOnePage({required double pageWidth, required double pageHeight}) {
    _pageColumns = 1;
    const pad = 48.0;
    const gap = 24.0;
    final byWidth = (_viewportWidth - pad) / (pageWidth + gap);
    final byHeight = (_viewportHeight - pad) / pageHeight;
    setZoom(byWidth < byHeight ? byWidth : byHeight);
  }

  /// Fit two pages side-by-side in the current viewport.
  void zoomToMultiplePages({required double pageWidth, required double pageHeight}) {
    _pageColumns = 2;
    const pad = 64.0;
    const gap = 24.0;
    // Each page adds horizontal padding of [gap]; two columns → 2× gap total.
    final byWidth = (_viewportWidth - pad) / (pageWidth * 2 + gap * 2);
    final byHeight = (_viewportHeight - pad) / pageHeight;
    setZoom(byWidth < byHeight ? byWidth : byHeight);
  }

  void toggleSplitView() {
    _splitView = !_splitView;
    notifyListeners();
  }

  void setSplitView(bool value) {
    if (_splitView == value) return;
    _splitView = value;
    notifyListeners();
  }

  void toggleRuler() {
    _showRuler = !_showRuler;
    notifyListeners();
  }

  /// Home → Show/Hide ¶ (non-printing characters).
  void toggleFormattingMarks() {
    _showFormattingMarks = !_showFormattingMarks;
    notifyListeners();
  }

  void setShowFormattingMarks(bool value) {
    if (_showFormattingMarks == value) return;
    _showFormattingMarks = value;
    notifyListeners();
  }

  void toggleNavigationPane() {
    _showNavigationPane = !_showNavigationPane;
    notifyListeners();
  }

  /// Open the navigation pane on the Outline tab (F19.S2).
  void showNavigationOutline() {
    _showNavigationPane = true;
    _preferOutlineTab = true;
    notifyListeners();
  }

  bool takePreferOutlineTab() {
    final prefer = _preferOutlineTab;
    _preferOutlineTab = false;
    return prefer;
  }

  void toggleStyleInspector() {
    _showStyleInspector = !_showStyleInspector;
    notifyListeners();
  }

  void showAccessibilityCheckerPane() {
    _showAccessibilityChecker = true;
    notifyListeners();
  }

  void hideAccessibilityCheckerPane() {
    if (!_showAccessibilityChecker) return;
    _showAccessibilityChecker = false;
    notifyListeners();
  }

  void requestScrollToPage(int page) {
    _scrollRequestPage = page;
    _caretScrollRequest = null;
    notifyListeners();
  }

  /// Queue a caret-follow scroll. Does not notify — the caller already did, or
  /// [DocumentView] drains this at the end of the frame.
  void requestScrollToCaret({
    required int page,
    required double y,
    required double height,
  }) {
    _caretScrollRequest = CaretScrollRequest(page: page, y: y, height: height);
    _scrollRequestPage = null;
  }

  int? takeScrollRequest() {
    final page = _scrollRequestPage;
    _scrollRequestPage = null;
    return page;
  }

  CaretScrollRequest? takeCaretScrollRequest() {
    final request = _caretScrollRequest;
    _caretScrollRequest = null;
    return request;
  }

  /// Every document open starts at 100%; zoom never carries over from the
  /// previous document.
  void reset() {
    _zoom = 1.0;
    _currentPage = 0;
    _printPreview = false;
    _layout = DocumentViewLayout.printLayout;
    _pageColumns = 1;
    _splitView = false;
    _scrollRequestPage = null;
    _caretScrollRequest = null;
    notifyListeners();
  }
}

/// One-shot request to scroll the document canvas so the caret stays visible.
class CaretScrollRequest {
  const CaretScrollRequest({
    required this.page,
    required this.y,
    required this.height,
  });

  final int page;
  final double y;
  final double height;
}
