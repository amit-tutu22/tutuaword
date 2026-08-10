import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/editor_menu.dart';
import 'package:tutuaword/ui/document_properties_dialog.dart';
import 'package:tutuaword/ui/find_pane.dart';
import 'package:tutuaword/ui/info_bar.dart';
import 'package:tutuaword/ui/password_dialog.dart';
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

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _controller.setPasswordPrompt(({fileName, errorMessage}) {
      if (!mounted) return Future<String?>.value(null);
      return PasswordDialog.show(
        context,
        fileName: fileName,
        errorMessage: errorMessage,
      );
    });
  }

  void _onUpdate() => setState(() {});

  @override
  void dispose() {
    _controller.setPasswordPrompt(null);
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
      onNewFromTemplate: () =>
          unawaited(_controller.openNewFromTemplateDialog(context)),
      onOpen: () => _controller.openDocument(),
      onOpenRecent: (path) => _controller.openRecentDocument(path),
      onSave: () => _controller.saveDocument(),
      onSaveAs: (ext) => _controller.saveDocumentAs(extension: ext),
      onSaveAsTemplate: () =>
          unawaited(_controller.openSaveAsTemplateDialog(context)),
      onPrint: () => unawaited(_controller.printDocument(context: context)),
      onShowProperties: () => DocumentPropertiesDialog.show(
        context,
        _controller.documentProperties,
      ),
      onShowPasteSpecial: () => _controller.showPasteSpecialDialog(context),
      onShowGoTo: () => unawaited(_controller.openGoToDialog(context)),
      onProtectWithPassword: () =>
          unawaited(_controller.protectWithPassword(context)),
      onRemovePassword: () =>
          unawaited(_controller.removePasswordProtection()),
      onInspectDocument: () =>
          unawaited(_controller.inspectDocument(context)),
      onDigitalSignatures: () =>
          unawaited(_controller.manageDigitalSignatures(context)),
      child: Material(
        color: WordTheme.tabStripSurface,
        child: Shortcuts(
          shortcuts: const <ShortcutActivator, Intent>{
            SingleActivator(LogicalKeyboardKey.keyF, meta: true): _OpenFindIntent(),
            SingleActivator(LogicalKeyboardKey.keyF, control: true): _OpenFindIntent(),
            SingleActivator(LogicalKeyboardKey.keyG, meta: true): _OpenGoToIntent(),
            SingleActivator(LogicalKeyboardKey.keyG, control: true): _OpenGoToIntent(),
            SingleActivator(LogicalKeyboardKey.keyA, meta: true): _SelectAllDocumentIntent(),
            SingleActivator(LogicalKeyboardKey.keyA, control: true): _SelectAllDocumentIntent(),
            SingleActivator(LogicalKeyboardKey.keyP, meta: true): _PrintDocumentIntent(),
            SingleActivator(LogicalKeyboardKey.keyP, control: true): _PrintDocumentIntent(),
          },
          child: Actions(
            actions: <Type, Action<Intent>>{
              _OpenFindIntent: CallbackAction<_OpenFindIntent>(
                onInvoke: (_) {
                  _controller.openFindPane();
                  return null;
                },
              ),
              _OpenGoToIntent: CallbackAction<_OpenGoToIntent>(
                onInvoke: (_) {
                  unawaited(_controller.openGoToDialog(context));
                  return null;
                },
              ),
              _SelectAllDocumentIntent: CallbackAction<_SelectAllDocumentIntent>(
                onInvoke: (_) {
                  unawaited(_controller.selectAll());
                  return null;
                },
              ),
              _PrintDocumentIntent: CallbackAction<_PrintDocumentIntent>(
                onInvoke: (_) {
                  unawaited(_controller.printDocument(context: context));
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

class _OpenGoToIntent extends Intent {
  const _OpenGoToIntent();
}

class _SelectAllDocumentIntent extends Intent {
  const _SelectAllDocumentIntent();
}

class _PrintDocumentIntent extends Intent {
  const _PrintDocumentIntent();
}
