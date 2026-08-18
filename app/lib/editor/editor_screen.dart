import 'dart:async';

import 'package:flutter/material.dart';
import 'package:tutuaword/editor/document_view.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/editor/editor_menu.dart';
import 'package:tutuaword/ui/document_properties_dialog.dart';
import 'package:tutuaword/ui/find_pane.dart';
import 'package:tutuaword/ui/info_bar.dart';
import 'package:tutuaword/ui/keyboard_shortcuts.dart';
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

  /// Word chord → controller command. One place, so the keyboard map and the
  /// ribbon always reach the same code.
  void _invokeChord(WordChord chord) {
    switch (chord) {
      case WordChord.newDocument:
        unawaited(_controller.newDocument());
      case WordChord.openDocument:
        unawaited(_controller.openDocument());
      case WordChord.save:
        unawaited(_controller.saveDocument());
      case WordChord.saveAs:
        unawaited(_controller.saveDocumentAs(extension: 'docx'));
      case WordChord.print:
        unawaited(_controller.printDocument(context: context));
      case WordChord.undo:
        unawaited(_controller.undo());
      case WordChord.redo:
        unawaited(_controller.redo());
      case WordChord.cut:
        unawaited(_controller.cutSelection());
      case WordChord.copy:
        unawaited(_controller.copySelection());
      case WordChord.paste:
        unawaited(_controller.paste());
      case WordChord.pasteTextOnly:
      case WordChord.pasteMatchStyle:
        unawaited(_controller.paste(plainText: true));
      case WordChord.selectAll:
        unawaited(_controller.selectAll());
      case WordChord.find:
      case WordChord.replace:
        _controller.openFindPane();
      case WordChord.findNext:
        _controller.findNext();
      case WordChord.goTo:
        unawaited(_controller.openGoToDialog(context));
      case WordChord.formattingMarks:
        _controller.toggleFormattingMarks();
      case WordChord.bold:
        _controller.toggleBold();
      case WordChord.italic:
        _controller.toggleItalic();
      case WordChord.underline:
        _controller.toggleUnderline();
      case WordChord.growFont:
        _controller.increaseFontSize();
      case WordChord.shrinkFont:
        _controller.decreaseFontSize();
      case WordChord.subscript:
        _controller.toggleSubscript();
      case WordChord.superscript:
        _controller.toggleSuperscript();
      case WordChord.allCaps:
        _controller.toggleAllCaps();
      case WordChord.smallCaps:
        _controller.toggleSmallCaps();
      case WordChord.clearFormatting:
        _controller.clearFormatting();
      case WordChord.alignLeft:
        _controller.setAlignment(TextAlign.left);
      case WordChord.alignCenter:
        _controller.setAlignment(TextAlign.center);
      case WordChord.alignRight:
        _controller.setAlignment(TextAlign.right);
      case WordChord.alignJustify:
        _controller.setAlignment(TextAlign.justify);
      case WordChord.increaseIndent:
        _controller.increaseIndent();
      case WordChord.decreaseIndent:
        _controller.decreaseIndent();
      case WordChord.singleSpace:
        _controller.applyLineSpacing(LineSpacingMode.single);
      case WordChord.oneAndAHalfSpace:
        _controller.applyLineSpacing(LineSpacingMode.oneAndHalf);
      case WordChord.doubleSpace:
        _controller.applyLineSpacing(LineSpacingMode.double_);
      case WordChord.normalStyle:
        _controller.applyNormalStyle();
      case WordChord.heading1:
        _controller.applyParagraphStyle('Heading 1');
      case WordChord.heading2:
        _controller.applyParagraphStyle('Heading 2');
      case WordChord.heading3:
        _controller.applyParagraphStyle('Heading 3');
      case WordChord.hyperlink:
        unawaited(_controller.insertHyperlink(context));
      case WordChord.comment:
        unawaited(_controller.insertComment(context));
      case WordChord.trackChanges:
        _controller.toggleTrackChanges();
    }
  }

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
          shortcuts: kWordChordShortcuts,
          child: Actions(
            actions: <Type, Action<Intent>>{
              WordChordIntent: CallbackAction<WordChordIntent>(
                onInvoke: (intent) {
                  _invokeChord(intent.chord);
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

