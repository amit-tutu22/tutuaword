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
      onShowAbout: () => unawaited(_controller.openAboutDialog(context)),
      onShowSettings: () => unawaited(_controller.openSettingsDialog(context)),
      child: Material(
        color: WordTheme.chrome(context).tabStrip,
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
            SingleActivator(LogicalKeyboardKey.keyS, meta: true): _SaveDocumentIntent(),
            SingleActivator(LogicalKeyboardKey.keyS, control: true): _SaveDocumentIntent(),
            SingleActivator(LogicalKeyboardKey.keyX, meta: true): _CutIntent(),
            SingleActivator(LogicalKeyboardKey.keyX, control: true): _CutIntent(),
            SingleActivator(LogicalKeyboardKey.keyC, meta: true): _CopyIntent(),
            SingleActivator(LogicalKeyboardKey.keyC, control: true): _CopyIntent(),
            SingleActivator(LogicalKeyboardKey.keyV, meta: true): _PasteIntent(),
            SingleActivator(LogicalKeyboardKey.keyV, control: true): _PasteIntent(),
            SingleActivator(LogicalKeyboardKey.keyV, meta: true, shift: true, alt: true):
                _PasteMatchStyleIntent(),
            SingleActivator(LogicalKeyboardKey.keyV, control: true, shift: true, alt: true):
                _PasteMatchStyleIntent(),
            SingleActivator(LogicalKeyboardKey.digit8, control: true, shift: true):
                _ToggleFormattingMarksIntent(),
            SingleActivator(LogicalKeyboardKey.digit8, meta: true, shift: true):
                _ToggleFormattingMarksIntent(),
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
              _SaveDocumentIntent: CallbackAction<_SaveDocumentIntent>(
                onInvoke: (_) {
                  unawaited(_controller.saveDocument());
                  return null;
                },
              ),
              _CutIntent: CallbackAction<_CutIntent>(
                onInvoke: (_) {
                  unawaited(_controller.cutSelection());
                  return null;
                },
              ),
              _CopyIntent: CallbackAction<_CopyIntent>(
                onInvoke: (_) {
                  unawaited(_controller.copySelection());
                  return null;
                },
              ),
              _PasteIntent: CallbackAction<_PasteIntent>(
                onInvoke: (_) {
                  unawaited(_controller.paste());
                  return null;
                },
              ),
              _PasteMatchStyleIntent: CallbackAction<_PasteMatchStyleIntent>(
                onInvoke: (_) {
                  unawaited(_controller.paste(plainText: true));
                  return null;
                },
              ),
              _ToggleFormattingMarksIntent:
                  CallbackAction<_ToggleFormattingMarksIntent>(
                onInvoke: (_) {
                  _controller.toggleFormattingMarks();
                  return null;
                },
              ),
            },
            child: Column(
              children: [
                if (!_controller.isReadMode)
                  WordTitleBar(
                    controller: _controller,
                    onHomePressed: () =>
                        _ribbonKey.currentState?.selectTab(RibbonTab.home),
                  ),
                if (!_controller.isReadMode)
                  WordRibbon(key: _ribbonKey, controller: _controller),
                if (!_controller.isReadMode && _controller.findPaneVisible)
                  FindPane(controller: _controller),
                if (!_controller.isReadMode)
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

class _SaveDocumentIntent extends Intent {
  const _SaveDocumentIntent();
}

class _CutIntent extends Intent {
  const _CutIntent();
}

class _CopyIntent extends Intent {
  const _CopyIntent();
}

class _PasteIntent extends Intent {
  const _PasteIntent();
}

class _PasteMatchStyleIntent extends Intent {
  const _PasteMatchStyleIntent();
}

class _ToggleFormattingMarksIntent extends Intent {
  const _ToggleFormattingMarksIntent();
}
