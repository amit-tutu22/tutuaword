import 'package:flutter/foundation.dart';
import 'package:tutuaword/bridge/document_engine.dart';
import 'package:tutuaword/bridge/find_format_filter.dart';
import 'package:tutuaword/bridge/find_match.dart';
import 'package:tutuaword/editor/controllers/engine_host.dart';
import 'package:tutuaword/editor/controllers/selection_controller.dart';
import 'package:tutuaword/editor/doc_range.dart';

typedef FindChangedCallback = void Function();

/// Find pane state, match list, and next/previous navigation (F18.S1).
class FindController extends ChangeNotifier {
  FindController({
    required EngineHost host,
    required SelectionController selection,
    required this.onFindChanged,
  })  : _host = host,
        _selection = selection;

  final EngineHost _host;
  final SelectionController _selection;
  final FindChangedCallback onFindChanged;

  bool _paneVisible = false;
  String _query = '';
  String _replaceText = '';
  bool _matchCase = false;
  bool _useRegex = false;
  bool _useWildcards = false;
  bool _findBold = false;
  String _findStyleName = '';
  List<FindMatch> _matches = const [];
  int _currentIndex = -1;
  String _statusText = '';

  bool get paneVisible => _paneVisible;
  String get query => _query;
  String get replaceText => _replaceText;
  bool get matchCase => _matchCase;
  bool get useRegex => _useRegex;
  bool get useWildcards => _useWildcards;
  bool get findBold => _findBold;
  String get findStyleName => _findStyleName;
  FindFormatFilter get formatFilter => FindFormatFilter(
        bold: _findBold ? true : null,
        styleName: _findStyleName.isEmpty ? null : _findStyleName,
      );
  List<FindMatch> get matches => _matches;
  int get currentIndex => _currentIndex;
  String get statusText => _statusText;

  DocumentEngine? get _engine => _host.engine;

  void openPane({String? initialQuery}) {
    _paneVisible = true;
    if (initialQuery != null && initialQuery.isNotEmpty) {
      _query = initialQuery;
    } else if (_selection.hasGlyphSelection) {
      final selected = _selection.selectedText().trim();
      if (selected.isNotEmpty && !selected.contains('\n')) {
        _query = selected;
      }
    }
    refreshMatches(selectFirst: true);
    notifyListeners();
  }

  void closePane() {
    _paneVisible = false;
    _currentIndex = -1;
    _statusText = '';
    _selection.collapseToCaret();
    onFindChanged();
    notifyListeners();
  }

  void setQuery(String value) {
    _query = value;
    refreshMatches(selectFirst: value.isNotEmpty);
  }

  void setReplaceText(String value) {
    _replaceText = value;
    notifyListeners();
  }

  Future<int?> replaceAll() async {
    if (_engine == null || _query.isEmpty) return null;
    final count = await _engine!.replaceAll(
      _query,
      _replaceText,
      _matchCase,
      useRegex: _useRegex,
      useWildcards: _useWildcards,
    );
    if (count == null) {
      _statusText = 'Replace failed';
    } else if (count == 0) {
      _statusText = 'No matches';
      _matches = const [];
      _currentIndex = -1;
    } else {
      refreshMatches(selectFirst: false);
      _statusText = 'Replaced $count occurrence(s)';
    }
    onFindChanged();
    notifyListeners();
    return count;
  }

  void toggleMatchCase() {
    _matchCase = !_matchCase;
    refreshMatches(selectFirst: _query.isNotEmpty);
  }

  void toggleUseRegex() {
    _useRegex = !_useRegex;
    if (_useRegex) {
      _useWildcards = false;
    }
    refreshMatches(selectFirst: _query.isNotEmpty);
  }

  void toggleUseWildcards() {
    _useWildcards = !_useWildcards;
    if (_useWildcards) {
      _useRegex = false;
    }
    refreshMatches(selectFirst: _query.isNotEmpty || _findBold || _findStyleName.isNotEmpty);
  }

  void toggleFindBold() {
    _findBold = !_findBold;
    refreshMatches(selectFirst: _query.isNotEmpty || _findBold || _findStyleName.isNotEmpty);
  }

  void setFindStyleName(String value) {
    _findStyleName = value;
    refreshMatches(selectFirst: _query.isNotEmpty || _findStyleName.isNotEmpty || _findBold);
  }

  void refreshMatches({bool selectFirst = false}) {
    final format = formatFilter;
    if (_engine == null || (_query.isEmpty && !format.isActive)) {
      _matches = const [];
      _currentIndex = -1;
      _statusText = _query.isEmpty && !format.isActive ? '' : 'No matches';
      onFindChanged();
      notifyListeners();
      return;
    }

    final found = _engine!.findMatches(
      _query,
      _matchCase,
      useRegex: _useRegex,
      useWildcards: _useWildcards,
      formatFilter: format,
    );
    if (found == null) {
      _matches = const [];
      _currentIndex = -1;
      _statusText = 'Invalid pattern';
      onFindChanged();
      notifyListeners();
      return;
    }
    _matches = found;
    if (_matches.isEmpty) {
      _currentIndex = -1;
      _statusText = 'No matches';
    } else if (selectFirst || _currentIndex < 0 || _currentIndex >= _matches.length) {
      _currentIndex = 0;
      _selectCurrentMatch();
      _statusText = '1 of ${_matches.length}';
    } else {
      _selectCurrentMatch();
      _statusText = '${_currentIndex + 1} of ${_matches.length}';
    }
    onFindChanged();
    notifyListeners();
  }

  void findNext() {
    if (_matches.isEmpty) {
      refreshMatches();
      return;
    }
    _currentIndex = (_currentIndex + 1) % _matches.length;
    _selectCurrentMatch();
    _statusText = '${_currentIndex + 1} of ${_matches.length}';
    onFindChanged();
    notifyListeners();
  }

  void findPrevious() {
    if (_matches.isEmpty) {
      refreshMatches();
      return;
    }
    _currentIndex = (_currentIndex - 1 + _matches.length) % _matches.length;
    _selectCurrentMatch();
    _statusText = '${_currentIndex + 1} of ${_matches.length}';
    onFindChanged();
    notifyListeners();
  }

  void _selectCurrentMatch() {
    if (_currentIndex < 0 || _currentIndex >= _matches.length) return;
    final match = _matches[_currentIndex];
    _selection.selectDocRange(
      DocRange(
        anchor: DocPosition(runId: match.startRunId, offset: match.start),
        focus: DocPosition(runId: match.endRunId, offset: match.end),
        page: _selection.caretPage,
      ),
    );
  }
}
