import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/document_session_store.dart';
import 'package:tutuaword/ui/app_chrome_theme.dart';

/// Persisted app chrome accent (title bar, tabs, ribbon).
class AppThemeController extends ChangeNotifier {
  AppThemeController({DocumentSessionStore? store})
      : _store = store ?? DocumentSessionStore.defaultStore() {
    _accent = _store.loadChromeAccent() ?? kDefaultChromeAccent;
  }

  final DocumentSessionStore _store;
  late Color _accent;

  static AppThemeController? _shared;
  static AppThemeController get instance => _shared ??= AppThemeController();

  @visibleForTesting
  static void debugReset() {
    _shared = null;
  }

  Color get accent => _accent;

  TutuawordChrome get chrome => TutuawordChrome.fromAccent(_accent);

  Future<void> setAccent(Color color) async {
    if (colorsEqual(color, _accent)) return;
    _accent = color;
    notifyListeners();
    await _store.saveChromeAccent(color);
  }
}
