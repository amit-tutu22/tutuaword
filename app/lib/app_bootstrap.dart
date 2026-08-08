import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/engine_bootstrap.dart';
import 'package:tutuaword/editor/editor_screen.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Boots the document engine off the critical path so Flutter can paint first.
///
/// Shows a lightweight loading shell until fonts and startup layout are ready,
/// then mounts [EditorScreen]. Mounting the editor earlier lets it anchor the
/// caret against an unfinished engine.
class AppBootstrap extends StatefulWidget {
  const AppBootstrap({super.key});

  @override
  State<AppBootstrap> createState() => _AppBootstrapState();
}

class _AppBootstrapState extends State<AppBootstrap> {
  bool _showEditor = false;
  Object? _error;

  @override
  void initState() {
    super.initState();
    // Paint the loading shell first so the native splash can dismiss, then
    // warm the engine before constructing EditorController.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      unawaited(_warmEngine());
    });
  }

  Future<void> _warmEngine() async {
    debugPrint('AppBootstrap: warming engine…');
    try {
      await warmDocumentEngine();
      debugPrint('AppBootstrap: engine ready');
      if (!mounted) return;
      setState(() => _showEditor = true);
    } catch (e, st) {
      debugPrint('AppBootstrap: engine init failed: $e\n$st');
      if (!mounted) return;
      setState(() => _error = e);
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_error != null) {
      return Scaffold(
        backgroundColor: WordTheme.tabStripSurface,
        body: Center(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Text(
              'Engine failed to start.\n$_error',
              textAlign: TextAlign.center,
            ),
          ),
        ),
      );
    }

    if (!_showEditor) {
      return _loadingScaffold('Starting engine…');
    }

    return const EditorScreen();
  }

  Widget _loadingScaffold(String message) {
    return Scaffold(
      backgroundColor: WordTheme.canvasGray,
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const CircularProgressIndicator(),
            const SizedBox(height: 16),
            Text(message),
          ],
        ),
      ),
    );
  }
}
