import 'dart:io' if (dart.library.html) 'package:tutuaword/bridge/platform_stub.dart';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:path/path.dart' as p;
import 'package:tutuaword/editor/editor_controller.dart';

/// Full macOS menu bar — app menu (Quit), File, Edit, View, Window.
class EditorMenuBar extends StatelessWidget {
  const EditorMenuBar({
    super.key,
    required this.controller,
    required this.onNew,
    required this.onNewFromTemplate,
    required this.onOpen,
    required this.onOpenRecent,
    required this.onSave,
    required this.onSaveAs,
    required this.onSaveAsTemplate,
    required this.onPrint,
    required this.onShowProperties,
    required this.onShowPasteSpecial,
    required this.onShowGoTo,
    required this.onProtectWithPassword,
    required this.onRemovePassword,
    required this.onInspectDocument,
    required this.onDigitalSignatures,
    required this.onShowAbout,
    required this.onShowSettings,
    required this.child,
  });

  final EditorController controller;
  final VoidCallback onNew;
  final VoidCallback onNewFromTemplate;
  final VoidCallback onOpen;
  final void Function(String path) onOpenRecent;
  final VoidCallback onSave;
  final void Function(String extension) onSaveAs;
  final VoidCallback onSaveAsTemplate;
  final VoidCallback onPrint;
  final VoidCallback onShowProperties;
  final VoidCallback onShowPasteSpecial;
  final VoidCallback onShowGoTo;
  final VoidCallback onProtectWithPassword;
  final VoidCallback onRemovePassword;
  final VoidCallback onInspectDocument;
  final VoidCallback onDigitalSignatures;
  final VoidCallback onShowAbout;
  final VoidCallback onShowSettings;
  final Widget child;

  static const _appName = 'tutuaword';

  PlatformMenuItemGroup? _group(List<PlatformMenuItem> members) {
    if (members.isEmpty) return null;
    return PlatformMenuItemGroup(members: members);
  }

  List<PlatformMenuItem> _compact(Iterable<PlatformMenuItem?> items) {
    return items.whereType<PlatformMenuItem>().toList();
  }

  @override
  Widget build(BuildContext context) {
    if (!Platform.isMacOS) {
      return child;
    }

    return PlatformMenuBar(
      menus: [
        _appMenu(context),
        _fileMenu(),
        _editMenu(),
        _toolsMenu(),
        _viewMenu(),
        _helpMenu(),
        _windowMenu(),
      ],
      child: child,
    );
  }

  PlatformMenu _appMenu(BuildContext context) {
    return PlatformMenu(
      label: _appName,
      menus: _compact([
        PlatformMenuItemGroup(
          members: [
            PlatformMenuItem(
              label: 'About Tutuaword',
              onSelected: onShowAbout,
            ),
            PlatformMenuItem(
              label: 'Settings…',
              shortcut: const SingleActivator(LogicalKeyboardKey.comma, meta: true),
              onSelected: onShowSettings,
            ),
          ],
        ),
        if (_has(PlatformProvidedMenuItemType.servicesSubmenu))
          PlatformMenuItemGroup(
            members: [
              PlatformProvidedMenuItem(
                type: PlatformProvidedMenuItemType.servicesSubmenu,
              ),
            ],
          ),
        _group([
          if (_has(PlatformProvidedMenuItemType.hide))
            PlatformProvidedMenuItem(
              type: PlatformProvidedMenuItemType.hide,
            ),
          if (_has(PlatformProvidedMenuItemType.hideOtherApplications))
            PlatformProvidedMenuItem(
              type: PlatformProvidedMenuItemType.hideOtherApplications,
            ),
          if (_has(PlatformProvidedMenuItemType.showAllApplications))
            PlatformProvidedMenuItem(
              type: PlatformProvidedMenuItemType.showAllApplications,
            ),
        ]),
        _group([
          if (_has(PlatformProvidedMenuItemType.quit))
            PlatformProvidedMenuItem(
              type: PlatformProvidedMenuItemType.quit,
            ),
        ]),
      ]),
    );
  }

  PlatformMenu _fileMenu() {
    return PlatformMenu(
      label: 'File',
      menus: [
        PlatformMenuItem(
          label: 'New',
          shortcut: const SingleActivator(LogicalKeyboardKey.keyN, meta: true),
          onSelected: onNew,
        ),
        PlatformMenuItem(
          label: 'New from Template…',
          onSelected: onNewFromTemplate,
        ),
        PlatformMenuItem(
          label: 'Open…',
          shortcut: const SingleActivator(LogicalKeyboardKey.keyO, meta: true),
          onSelected: onOpen,
        ),
        if (controller.recentDocuments.isNotEmpty)
          PlatformMenuItemGroup(
            members: [
              for (final path in controller.recentDocuments)
                PlatformMenuItem(
                  label: p.basename(path),
                  onSelected: () => onOpenRecent(path),
                ),
            ],
          ),
        PlatformMenuItem(
          label: 'Save',
          shortcut: const SingleActivator(LogicalKeyboardKey.keyS, meta: true),
          onSelected: onSave,
        ),
        PlatformMenuItem(
          label: 'Save as Template…',
          onSelected: onSaveAsTemplate,
        ),
        PlatformMenuItem(
          label: 'Print…',
          shortcut: const SingleActivator(LogicalKeyboardKey.keyP, meta: true),
          onSelected: onPrint,
        ),
        PlatformMenuItem(
          label: 'Properties…',
          onSelected: onShowProperties,
        ),
        PlatformMenuItem(
          label: 'Inspect Document…',
          onSelected: onInspectDocument,
        ),
        PlatformMenuItem(
          label: 'Digital Signatures…',
          onSelected: onDigitalSignatures,
        ),
        PlatformMenuItemGroup(
          members: [
            PlatformMenuItem(
              label: 'Protect with Password…',
              onSelected: onProtectWithPassword,
            ),
            if (controller.encryptionPasswordSet)
              PlatformMenuItem(
                label: 'Remove Password',
                onSelected: onRemovePassword,
              ),
          ],
        ),
        PlatformMenuItemGroup(
          members: [
            PlatformMenuItem(
              label: 'Save as TWDOC…',
              onSelected: () => onSaveAs('twdoc'),
            ),
            PlatformMenuItem(
              label: 'Save as DOCX…',
              shortcut: const SingleActivator(
                LogicalKeyboardKey.keyS,
                meta: true,
                shift: true,
              ),
              onSelected: () => onSaveAs('docx'),
            ),
            PlatformMenuItem(
              label: 'Save as ODT…',
              onSelected: () => onSaveAs('odt'),
            ),
            PlatformMenuItem(
              label: 'Save as Markdown…',
              onSelected: () => onSaveAs('md'),
            ),
            PlatformMenuItem(
              label: 'Save as HTML…',
              onSelected: () => onSaveAs('html'),
            ),
            PlatformMenuItem(
              label: 'Export PDF…',
              onSelected: controller.exportPdf,
            ),
          ],
        ),
      ],
    );
  }

  PlatformMenu _toolsMenu() {
    return PlatformMenu(
      label: 'Tools',
      menus: [
        PlatformMenuItem(
          label: 'Spell Check',
          onSelected: controller.spellCheckDocument,
        ),
        PlatformMenuItem(
          label: controller.trackChanges ? 'Turn Off Track Changes' : 'Turn On Track Changes',
          onSelected: controller.toggleTrackChanges,
        ),
      ],
    );
  }

  PlatformMenu _editMenu() {
    return PlatformMenu(
      label: 'Edit',
      menus: [
        PlatformMenuItem(
          label: 'Undo',
          shortcut: const SingleActivator(LogicalKeyboardKey.keyZ, meta: true),
          onSelected: controller.undo,
        ),
        PlatformMenuItem(
          label: 'Redo',
          shortcut: const SingleActivator(
            LogicalKeyboardKey.keyZ,
            meta: true,
            shift: true,
          ),
          onSelected: controller.redo,
        ),
        PlatformMenuItemGroup(
          members: [
            PlatformMenuItem(
              label: 'Cut',
              shortcut: const SingleActivator(LogicalKeyboardKey.keyX, meta: true),
              onSelected: controller.cutSelection,
            ),
            PlatformMenuItem(
              label: 'Copy',
              shortcut: const SingleActivator(LogicalKeyboardKey.keyC, meta: true),
              onSelected: controller.copySelection,
            ),
            PlatformMenuItem(
              label: 'Paste',
              shortcut: const SingleActivator(LogicalKeyboardKey.keyV, meta: true),
              onSelected: controller.paste,
            ),
            PlatformMenuItem(
              label: 'Paste and Match Style',
              shortcut: const SingleActivator(
                LogicalKeyboardKey.keyV,
                meta: true,
                shift: true,
                alt: true,
              ),
              onSelected: () => controller.paste(plainText: true),
            ),
            PlatformMenuItem(
              label: 'Paste Special…',
              onSelected: onShowPasteSpecial,
            ),
            PlatformMenuItem(
              label: 'Delete',
              onSelected: controller.deleteSelection,
            ),
            PlatformMenuItem(
              label: 'Select All',
              shortcut: const SingleActivator(LogicalKeyboardKey.keyA, meta: true),
              onSelected: controller.selectAll,
            ),
            PlatformMenuItem(
              label: 'Find…',
              shortcut: const SingleActivator(LogicalKeyboardKey.keyF, meta: true),
              onSelected: controller.openFindPane,
            ),
            PlatformMenuItem(
              label: 'Go To…',
              shortcut: const SingleActivator(LogicalKeyboardKey.keyG, meta: true),
              onSelected: onShowGoTo,
            ),
          ],
        ),
      ],
    );
  }

  PlatformMenu _helpMenu() {
    return PlatformMenu(
      label: 'Help',
      menus: [
        PlatformMenuItem(
          label: 'About Tutuaword',
          onSelected: onShowAbout,
        ),
      ],
    );
  }

  PlatformMenu _viewMenu() {
    return PlatformMenu(
      label: 'View',
      menus: _compact([
        PlatformMenuItem(
          label: controller.printPreview ? 'Exit Print Preview' : 'Print Preview',
          onSelected: controller.togglePrintPreview,
        ),
        if (_has(PlatformProvidedMenuItemType.toggleFullScreen))
          PlatformProvidedMenuItem(
            type: PlatformProvidedMenuItemType.toggleFullScreen,
          ),
      ]),
    );
  }

  PlatformMenu _windowMenu() {
    return PlatformMenu(
      label: 'Window',
      menus: _compact([
        if (_has(PlatformProvidedMenuItemType.minimizeWindow))
          PlatformProvidedMenuItem(
            type: PlatformProvidedMenuItemType.minimizeWindow,
          ),
        if (_has(PlatformProvidedMenuItemType.zoomWindow))
          PlatformProvidedMenuItem(
            type: PlatformProvidedMenuItemType.zoomWindow,
          ),
        _group([
          if (_has(PlatformProvidedMenuItemType.arrangeWindowsInFront))
            PlatformProvidedMenuItem(
              type: PlatformProvidedMenuItemType.arrangeWindowsInFront,
            ),
        ]),
      ]),
    );
  }

  bool _has(PlatformProvidedMenuItemType type) {
    return PlatformProvidedMenuItem.hasMenu(type);
  }
}
