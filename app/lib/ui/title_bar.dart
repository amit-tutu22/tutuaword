import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';
import 'package:tutuaword/ui/word_theme.dart';

class WordTitleBar extends StatelessWidget {
  const WordTitleBar({
    super.key,
    required this.controller,
    this.onHomePressed,
  });

  final EditorController controller;
  final VoidCallback? onHomePressed;

  @override
  Widget build(BuildContext context) {
    // iOS and Android draw the status bar, notch, and Dynamic Island over the
    // app. The blue extends under them so the bar still reads as one surface,
    // while the quick-access controls sit below the inset.
    final viewPadding = MediaQuery.paddingOf(context);
    final phone = WordTheme.phoneChrome(context);
    return AnnotatedRegion<SystemUiOverlayStyle>(
      // Light glyphs: the system clock and indicators sit on the blue bar.
      value: SystemUiOverlayStyle.light,
      child: Container(
        key: const Key('title_bar_chrome'),
        color: WordTheme.chrome(context).titleBar,
        padding: EdgeInsets.only(
          top: viewPadding.top,
          left: viewPadding.left,
          right: viewPadding.right,
        ),
        child: SizedBox(
          height: WordTheme.titleBarHeight,
          child: phone ? _buildPhoneBar(context) : _buildDesktopBar(context),
        ),
      ),
    );
  }

  /// Phone: title between icon groups; Undo / Redo / Print in overflow menu.
  Widget _buildPhoneBar(BuildContext context) {
    return Row(
      children: [
        SizedBox(width: WordTheme.leadingChromeInset(context)),
        _QuickAccessIcon(
          icon: Icons.home_outlined,
          tooltip: onHomePressed == null ? kComingSoonTooltip : 'Home',
          onPressed: onHomePressed,
        ),
        _QuickAccessIcon(
          icon: Icons.folder_open_outlined,
          tooltip: 'Open',
          onPressed: () => controller.openDocument(),
        ),
        _QuickAccessIcon(
          icon: Icons.save_outlined,
          tooltip: 'Save',
          onPressed: () => controller.saveDocument(),
        ),
        Expanded(
          child: Text(
            controller.documentTitle,
            style: WordTheme.titleBarTitle,
            overflow: TextOverflow.ellipsis,
            textAlign: TextAlign.center,
            maxLines: 1,
          ),
        ),
        _QuickAccessIcon(
          key: const Key('title_bar_search'),
          icon: Icons.search,
          tooltip: 'Find',
          onPressed: controller.openFindPane,
        ),
        _QuickAccessIcon(
          key: const Key('title_bar_overflow'),
          icon: Icons.more_horiz,
          tooltip: 'More actions',
          onPressed: () => _showOverflowMenu(context),
        ),
        const SizedBox(width: 12),
      ],
    );
  }

  void _showOverflowMenu(BuildContext context) {
    final box = context.findRenderObject() as RenderBox?;
    if (box == null) return;
    final overlay = Overlay.of(context).context.findRenderObject() as RenderBox;
    final origin = box.localToGlobal(Offset.zero, ancestor: overlay);
    showMenu<void>(
      context: context,
      position: RelativeRect.fromLTRB(
        origin.dx,
        origin.dy + box.size.height,
        origin.dx + box.size.width,
        origin.dy + box.size.height + 4,
      ),
      items: [
        PopupMenuItem<void>(
          onTap: controller.undo,
          child: const ListTile(
            dense: true,
            leading: Icon(Icons.undo, size: 18),
            title: Text('Undo', style: TextStyle(fontSize: 13)),
            contentPadding: EdgeInsets.zero,
          ),
        ),
        PopupMenuItem<void>(
          onTap: controller.redo,
          child: const ListTile(
            dense: true,
            leading: Icon(Icons.redo, size: 18),
            title: Text('Redo', style: TextStyle(fontSize: 13)),
            contentPadding: EdgeInsets.zero,
          ),
        ),
        PopupMenuItem<void>(
          onTap: () => controller.printDocument(context: context),
          child: const ListTile(
            dense: true,
            leading: Icon(Icons.print_outlined, size: 18),
            title: Text('Print', style: TextStyle(fontSize: 13)),
            contentPadding: EdgeInsets.zero,
          ),
        ),
        PopupMenuItem<void>(
          onTap: () {
            WidgetsBinding.instance.addPostFrameCallback((_) {
              if (context.mounted) {
                controller.openKeyboardHelpDialog(context);
              }
            });
          },
          child: const ListTile(
            dense: true,
            leading: Icon(Icons.keyboard_alt_outlined, size: 18),
            title: Text('Keyboard Shortcuts', style: TextStyle(fontSize: 13)),
            contentPadding: EdgeInsets.zero,
          ),
        ),
        PopupMenuItem<void>(
          onTap: () {
            WidgetsBinding.instance.addPostFrameCallback((_) {
              if (context.mounted) {
                controller.openAboutDialog(context);
              }
            });
          },
          child: const ListTile(
            dense: true,
            leading: Icon(Icons.info_outline, size: 18),
            title: Text('About Tutuaword', style: TextStyle(fontSize: 13)),
            contentPadding: EdgeInsets.zero,
          ),
        ),
        PopupMenuItem<void>(
          onTap: () {
            WidgetsBinding.instance.addPostFrameCallback((_) {
              if (context.mounted) {
                controller.openSettingsDialog(context);
              }
            });
          },
          child: const ListTile(
            dense: true,
            leading: Icon(Icons.settings_outlined, size: 18),
            title: Text('App Settings', style: TextStyle(fontSize: 13)),
            contentPadding: EdgeInsets.zero,
          ),
        ),
      ],
    );
  }

  /// Tablet + desktop: title centered on full bar; QAT floats above.
  Widget _buildDesktopBar(BuildContext context) {
    return Stack(
      alignment: Alignment.center,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 120),
          child: Text(
            controller.documentTitle,
            style: WordTheme.titleBarTitle,
            overflow: TextOverflow.ellipsis,
            textAlign: TextAlign.center,
            maxLines: 1,
          ),
        ),
        Row(
          children: [
            SizedBox(width: WordTheme.leadingChromeInset(context)),
            _QuickAccessIcon(
              icon: Icons.home_outlined,
              tooltip: onHomePressed == null ? kComingSoonTooltip : 'Home',
              onPressed: onHomePressed,
            ),
            _QuickAccessIcon(
              icon: Icons.folder_open_outlined,
              tooltip: 'Open',
              onPressed: () => controller.openDocument(),
            ),
            _QuickAccessIcon(
              icon: Icons.save_outlined,
              tooltip: 'Save',
              onPressed: () => controller.saveDocument(),
            ),
            _QuickAccessIcon(
              icon: Icons.undo,
              tooltip: 'Undo',
              onPressed: controller.undo,
            ),
            _QuickAccessIcon(
              icon: Icons.redo,
              tooltip: 'Redo',
              onPressed: controller.redo,
            ),
            _QuickAccessIcon(
              icon: Icons.print_outlined,
              tooltip: 'Print',
              onPressed: () => controller.printDocument(context: context),
            ),
            const Spacer(),
            _QuickAccessIcon(
              key: const Key('title_bar_search'),
              icon: Icons.search,
              tooltip: 'Find',
              onPressed: controller.openFindPane,
            ),
            _HelpSettingsButton(controller: controller),
            const SizedBox(width: 12),
          ],
        ),
      ],
    );
  }
}

/// Top-right gear: Keyboard Shortcuts, About, and App Settings.
class _HelpSettingsButton extends StatelessWidget {
  const _HelpSettingsButton({required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    return PopupMenuButton<String>(
      key: const Key('title_bar_settings'),
      tooltip: 'Help & settings',
      padding: EdgeInsets.zero,
      offset: const Offset(0, 28),
      onSelected: (value) {
        switch (value) {
          case 'keyboard':
            controller.openKeyboardHelpDialog(context);
          case 'about':
            controller.openAboutDialog(context);
          case 'settings':
            controller.openSettingsDialog(context);
        }
      },
      itemBuilder: (context) => [
        const PopupMenuItem(
          key: Key('title_bar_keyboard_help'),
          value: 'keyboard',
          child: ListTile(
            dense: true,
            leading: Icon(Icons.keyboard_alt_outlined, size: 18),
            title: Text('Keyboard Shortcuts', style: TextStyle(fontSize: 13)),
            contentPadding: EdgeInsets.zero,
          ),
        ),
        const PopupMenuItem(
          key: Key('title_bar_about'),
          value: 'about',
          child: ListTile(
            dense: true,
            leading: Icon(Icons.info_outline, size: 18),
            title: Text('About Tutuaword', style: TextStyle(fontSize: 13)),
            contentPadding: EdgeInsets.zero,
          ),
        ),
        const PopupMenuItem(
          key: Key('title_bar_app_settings'),
          value: 'settings',
          child: ListTile(
            dense: true,
            leading: Icon(Icons.settings_outlined, size: 18),
            title: Text('App Settings', style: TextStyle(fontSize: 13)),
            contentPadding: EdgeInsets.zero,
          ),
        ),
      ],
      child: const _QuickAccessIconFace(
        icon: Icons.settings_outlined,
      ),
    );
  }
}

/// Icon chrome shared by title-bar buttons (hover handled by parent menus).
class _QuickAccessIconFace extends StatelessWidget {
  const _QuickAccessIconFace({required this.icon});

  final IconData icon;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: 28,
      height: 28,
      child: Icon(icon, size: 16, color: WordTheme.titleBarIcon),
    );
  }
}

class _QuickAccessIcon extends StatefulWidget {
  const _QuickAccessIcon({
    super.key,
    required this.icon,
    required this.tooltip,
    this.onPressed,
  });

  final IconData icon;
  final String tooltip;
  final VoidCallback? onPressed;

  @override
  State<_QuickAccessIcon> createState() => _QuickAccessIconState();
}

class _QuickAccessIconState extends State<_QuickAccessIcon> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onPressed != null;
    return Tooltip(
      message: widget.tooltip,
      child: MouseRegion(
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: GestureDetector(
          onTap: widget.onPressed,
          child: Container(
            width: 28,
            height: 28,
            margin: const EdgeInsets.symmetric(horizontal: 1),
            decoration: BoxDecoration(
              color: _hovered && enabled
                  ? Colors.white.withValues(alpha: 0.15)
                  : Colors.transparent,
              borderRadius: BorderRadius.circular(3),
            ),
            child: Icon(
              widget.icon,
              size: 16,
              color: enabled ? WordTheme.titleBarIcon : WordTheme.titleBarIcon.withValues(alpha: 0.4),
            ),
          ),
        ),
      ),
    );
  }
}
