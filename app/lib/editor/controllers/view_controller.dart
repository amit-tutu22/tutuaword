import 'package:flutter/foundation.dart';

/// Zoom, page navigation, ruler, navigation pane, print preview.
class ViewController extends ChangeNotifier {
  double _zoom = 1.0;
  int _currentPage = 0;
  bool _printPreview = false;
  bool _showRuler = false;
  bool _showNavigationPane = false;
  bool _showStyleInspector = false;
  String _statusSuffix = '';
  int? _scrollRequestPage;

  double get zoom => _zoom;
  int get currentPage => _currentPage;
  bool get printPreview => _printPreview;
  bool get showRuler => _showRuler;
  bool get showNavigationPane => _showNavigationPane;
  bool get showStyleInspector => _showStyleInspector;
  String get statusSuffix => _statusSuffix;
  int? get scrollRequestPage => _scrollRequestPage;

  void setStatusSuffix(String value) {
    _statusSuffix = value;
  }

  void setCurrentPage(int page, int pageCount) {
    _currentPage = page.clamp(0, pageCount - 1);
    notifyListeners();
  }

  void setVisiblePage(int page, int pageCount) {
    final clamped = page.clamp(0, pageCount - 1);
    if (clamped == _currentPage) return;
    _currentPage = clamped;
    notifyListeners();
  }

  void togglePrintPreview() {
    _printPreview = !_printPreview;
    notifyListeners();
  }

  void setZoom(double value) {
    _zoom = value.clamp(0.5, 3.0);
    notifyListeners();
  }

  void zoomIn() => setZoom(_zoom + 0.1);
  void zoomOut() => setZoom(_zoom - 0.1);

  void toggleRuler() {
    _showRuler = !_showRuler;
    notifyListeners();
  }

  void toggleNavigationPane() {
    _showNavigationPane = !_showNavigationPane;
    notifyListeners();
  }

  void toggleStyleInspector() {
    _showStyleInspector = !_showStyleInspector;
    notifyListeners();
  }

  void requestScrollToPage(int page) {
    _scrollRequestPage = page;
    notifyListeners();
  }

  int? takeScrollRequest() {
    final page = _scrollRequestPage;
    _scrollRequestPage = null;
    return page;
  }

  void reset() {
    _currentPage = 0;
    _printPreview = false;
    notifyListeners();
  }
}
