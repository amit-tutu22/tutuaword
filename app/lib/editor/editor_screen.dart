import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/editor_menu.dart';
import 'package:tutuaword/ui/document_properties_dialog.dart';
import 'package:tutuaword/ui/find_pane.dart';
import 'package:tutuaword/ui/info_bar.dart';
import 'package:tutuaword/ui/ribbon.dart';
import 'package:tutuaword/ui/status_bar.dart';
import 'package:tutuaword/ui/title_bar.dart';
import 'package:tutuaword/ui/word_theme.dart';

class EditorScreen extends StatefulWidget {
  const EditorScreen({super.key, this.controller});

  /// When set (e.g. in widget tests), this controller is used instead of
  /// creating a production [EditorController] with autosave enabled.
  final EditorController? controller;

  @override
  State<EditorScreen> createState() => _EditorScreenState();
}

class _EditorScreenState extends State<EditorScreen> {
  late final EditorController _controller;
  late final bool _ownsController;
  final _ribbonKey = GlobalKey<WordRibbonState>();

  @override
  void initState() {
    super.initState();
    if (widget.controller != null) {
      _controller = widget.controller!;
      _ownsController = false;
    } else {
      _controller = EditorController();
      _ownsController = true;
    }
    _controller.addListener(_onUpdate);
    if (_ownsController) {
      WidgetsBinding.instance.addPostFrameCallback((_) async {
        await _controller.tryRecoverAutosave();
      });
    }
  }

  void _onUpdate() => setState(() {});

  @override
  void dispose() {
    _controller.removeListener(_onUpdate);
    if (_ownsController) {
      _controller.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return EditorMenuBar(
      controller: _controller,
      onNew: () => _controller.newDocument(),
      onOpen: () => _controller.openDocument(),
      onOpenRecent: (path) => _controller.openRecentDocument(path),
      onSave: () => _controller.saveDocument(),
      onSaveAs: (ext) => _controller.saveDocumentAs(extension: ext),
      onShowProperties: () => DocumentPropertiesDialog.show(
        context,
        _controller.documentProperties,
      ),
      onShowPasteSpecial: () => _controller.showPasteSpecialDialog(context),
      child: Material(
        color: WordTheme.tabStripSurface,
        child: Shortcuts(
          shortcuts: const <ShortcutActivator, Intent>{
            SingleActivator(LogicalKeyboardKey.keyF, meta: true): _OpenFindIntent(),
            SingleActivator(LogicalKeyboardKey.keyF, control: true): _OpenFindIntent(),
            SingleActivator(LogicalKeyboardKey.keyA, meta: true): _SelectAllDocumentIntent(),
            SingleActivator(LogicalKeyboardKey.keyA, control: true): _SelectAllDocumentIntent(),
          },
          child: Actions(
            actions: <Type, Action<Intent>>{
              _OpenFindIntent: CallbackAction<_OpenFindIntent>(
                onInvoke: (_) {
                  _controller.openFindPane();
                  return null;
                },
              ),
              _SelectAllDocumentIntent: CallbackAction<_SelectAllDocumentIntent>(
                onInvoke: (_) {
                  unawaited(_controller.selectAll());
                  return null;
                },
              ),
            },
            child: Column(
              children: [
                WordTitleBar(
                  controller: _controller,
                  onHomePressed: () =>
                      _ribbonKey.currentState?.selectTab(RibbonTab.home),
                ),
                WordRibbon(key: _ribbonKey, controller: _controller),
                if (_controller.findPaneVisible)
                  FindPane(controller: _controller),
                InfoBar(controller: _controller),
                Expanded(
                  child: DocumentView(controller: _controller),
                ),
                WordStatusBar(controller: _controller),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _OpenFindIntent extends Intent {
  const _OpenFindIntent();
}

class _SelectAllDocumentIntent extends Intent {
  const _SelectAllDocumentIntent();
}
