import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/engine_bootstrap.dart';
import 'package:tutuaword/editor/editor_screen.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Boots the document engine off the critical path so Flutter can paint first.
///
/// Shows a lightweight loading shell until fonts and startup layout are ready,
/// then mounts [EditorScreen]. If the engine cannot load, keeps the editor
/// (and ribbon) unmounted and shows a recoverable error instead of a
/// disconnected no-op UI.
class AppBootstrap extends StatefulWidget {
  const AppBootstrap({
    super.key,
    this.warmEngine,
    this.editorBuilder,
  });

  /// Override for tests; defaults to [warmDocumentEngine].
  final Future<void> Function()? warmEngine;

  /// Override for tests; defaults to [EditorScreen].
  final WidgetBuilder? editorBuilder;

  @override
  State<AppBootstrap> createState() => _AppBootstrapState();
}

class _AppBootstrapState extends State<AppBootstrap> {
  bool _showEditor = false;
  bool _warming = false;
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
    if (_warming) return;
    setState(() {
      _warming = true;
      _error = null;
      _showEditor = false;
    });
    debugPrint('AppBootstrap: warming engine…');
    try {
      await (widget.warmEngine ?? warmDocumentEngine)();
      debugPrint('AppBootstrap: engine ready');
      if (!mounted) return;
      setState(() {
        _showEditor = true;
        _warming = false;
      });
    } catch (e, st) {
      debugPrint('AppBootstrap: engine init failed: $e\n$st');
      if (!mounted) return;
      setState(() {
        _error = e;
        _warming = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_error != null) {
      return Scaffold(
        backgroundColor: WordTheme.chrome(context).tabStrip,
        body: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 480),
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(Icons.error_outline, size: 48, color: Colors.redAccent),
                  const SizedBox(height: 16),
                  const Text(
                    'Document engine failed to start',
                    textAlign: TextAlign.center,
                    style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
                  ),
                  const SizedBox(height: 12),
                  Text(
                    '$_error',
                    key: const Key('engine_bootstrap_error'),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 8),
                  const Text(
                    'The editor and ribbon stay disabled until the engine loads.',
                    textAlign: TextAlign.center,
                    style: TextStyle(color: Colors.black54),
                  ),
                  const SizedBox(height: 20),
                  FilledButton(
                    key: const Key('engine_bootstrap_retry'),
                    onPressed: _warming ? null : () => unawaited(_warmEngine()),
                    child: const Text('Retry'),
                  ),
                ],
              ),
            ),
          ),
        ),
      );
    }

    if (!_showEditor) {
      return _loadingScaffold('Starting engine…');
    }

    return widget.editorBuilder?.call(context) ?? const EditorScreen();
  }

  Widget _loadingScaffold(String message) {
    return Scaffold(
      backgroundColor: WordTheme.chrome(context).canvas,
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
