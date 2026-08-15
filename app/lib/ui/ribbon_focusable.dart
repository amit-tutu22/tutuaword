import 'package:flutter/material.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Focus + hover chrome for ribbon controls (F21.S2).
///
/// Enabled controls participate in Tab order and activate with Space/Enter.
/// Disabled controls are skipped by focus traversal.
class RibbonFocusable extends StatefulWidget {
  const RibbonFocusable({
    super.key,
    required this.enabled,
    required this.onActivate,
    required this.builder,
    this.focusNode,
    this.autofocus = false,
  });

  final bool enabled;
  final VoidCallback? onActivate;
  final Widget Function(
    BuildContext context, {
    required bool hovered,
    required bool focused,
  }) builder;
  final FocusNode? focusNode;
  final bool autofocus;

  @override
  State<RibbonFocusable> createState() => _RibbonFocusableState();
}

class _RibbonFocusableState extends State<RibbonFocusable> {
  bool _hovered = false;
  bool _focused = false;
  FocusNode? _ownedNode;

  FocusNode get _node => widget.focusNode ?? _ownedNode!;

  @override
  void initState() {
    super.initState();
    if (widget.focusNode == null) {
      _ownedNode = FocusNode(debugLabel: 'RibbonFocusable');
    }
  }

  @override
  void dispose() {
    _ownedNode?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FocusableActionDetector(
      focusNode: _node,
      autofocus: widget.autofocus,
      enabled: widget.enabled,
      mouseCursor: widget.enabled
          ? SystemMouseCursors.click
          : SystemMouseCursors.basic,
      onShowHoverHighlight: (show) {
        if (_hovered != show) setState(() => _hovered = show);
      },
      onShowFocusHighlight: (show) {
        if (_focused != show) setState(() => _focused = show);
      },
      actions: <Type, Action<Intent>>{
        ActivateIntent: CallbackAction<ActivateIntent>(
          onInvoke: (_) {
            widget.onActivate?.call();
            return null;
          },
        ),
      },
      // Pointer taps must invoke onActivate too — FocusableActionDetector only
      // wires ActivateIntent for keyboard (Space/Enter). Without this, macOS /
      // desktop mouse clicks on the ribbon appear dead.
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: widget.enabled ? widget.onActivate : null,
        child: widget.builder(
          context,
          hovered: _hovered,
          focused: _focused,
        ),
      ),
    );
  }
}

/// Standard focus ring used by ribbon controls.
BoxDecoration ribbonFocusDecoration({
  required Color fill,
  required bool focused,
  BorderRadius? borderRadius,
  Color? focusColor,
}) {
  final radius = borderRadius ?? BorderRadius.circular(3);
  return BoxDecoration(
    color: fill,
    borderRadius: radius,
    border: focused
        ? Border.all(
            color: focusColor ?? WordTheme.activeTabUnderline,
            width: 1.5,
          )
        : null,
  );
}
