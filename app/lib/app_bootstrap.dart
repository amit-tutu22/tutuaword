import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/engine_bootstrap.dart';
import 'package:tutuaword/editor/editor_screen.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Boots the document engine off the critical path so Flutter can paint first.
class AppBootstrap extends StatefulWidget {
  const AppBootstrap({super.key});

  @override
  State<AppBootstrap> createState() => _AppBootstrapState();
}

class _AppBootstrapState extends State<AppBootstrap> {
  bool _showEditor = false;
  Object? _error;
  String? _warmupStatus;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      // Paint the shell immediately so the native splash can dismiss.
      setState(() => _showEditor = true);
      unawaited(_warmEngine());
    });
  }

  Future<void> _warmEngine() async {
    debugPrint('AppBootstrap: warming engine…');
    if (mounted) {
      setState(() => _warmupStatus = 'Starting engine…');
    }
    try {
      await warmDocumentEngine();
      debugPrint('AppBootstrap: engine ready');
      if (!mounted) return;
      setState(() => _warmupStatus = null);
    } catch (e, st) {
      debugPrint('AppBootstrap: engine init failed: $e\n$st');
      if (!mounted) return;
      setState(() {
        _error = e;
        _warmupStatus = null;
      });
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
      return _loadingScaffold('Loading…');
    }

    return Stack(
      children: [
        const EditorScreen(),
        if (_warmupStatus != null)
          Positioned(
            left: 0,
            right: 0,
            top: 0,
            child: Material(
              color: WordTheme.infoBarAmber,
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    ),
                    const SizedBox(width: 12),
                    Text(_warmupStatus!),
                  ],
                ),
              ),
            ),
          ),
      ],
    );
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
