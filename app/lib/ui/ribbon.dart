import 'dart:async';

import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_focusable.dart';
import 'package:tutuaword/ui/ribbon_tabs/design_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/home_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/insert_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/layout_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/mailings_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/references_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/review_tab.dart';
import 'package:tutuaword/ui/ribbon_tabs/view_tab.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';
import 'package:tutuaword/ui/word_theme.dart';

enum RibbonTab {
  home,
  insert,
  design,
  layout,
  references,
  mailings,
  review,
  view,
}

extension RibbonTabLabel on RibbonTab {
  String get label => switch (this) {
        RibbonTab.home => 'Home',
        RibbonTab.insert => 'Insert',
        RibbonTab.design => 'Design',
        RibbonTab.layout => 'Layout',
        RibbonTab.references => 'References',
        RibbonTab.mailings => 'Mailings',
        RibbonTab.review => 'Review',
        RibbonTab.view => 'View',
      };
}

class WordRibbon extends StatefulWidget {
  const WordRibbon({super.key, required this.controller});

  final EditorController controller;

  @override
  State<WordRibbon> createState() => WordRibbonState();
}

class WordRibbonState extends State<WordRibbon> {
  RibbonTab _activeTab = RibbonTab.home;

  void selectTab(RibbonTab tab) => setState(() => _activeTab = tab);

  @override
  Widget build(BuildContext context) {
    // Reading-order traversal: tab strip L→R, then enabled controls in the
    // active tab body (F21.S2).
    return FocusTraversalGroup(
      key: const Key('word_ribbon'),
      policy: ReadingOrderTraversalPolicy(),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          _TabStrip(
            activeTab: _activeTab,
            onTabSelected: (tab) => setState(() => _activeTab = tab),
            onShare: () => unawaited(widget.controller.shareWithApps()),
          ),
          Container(
            height: WordTheme.ribbonHeight,
            color: WordTheme.ribbonSurface,
            child: ClipRect(
              child: _buildTabContent(),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildTabContent() {
    return switch (_activeTab) {
      RibbonTab.home => HomeTab(controller: widget.controller),
      RibbonTab.insert => InsertTab(controller: widget.controller),
      RibbonTab.design => DesignTab(controller: widget.controller),
      RibbonTab.layout => LayoutTab(controller: widget.controller),
      RibbonTab.references => ReferencesTab(controller: widget.controller),
      RibbonTab.mailings => MailingsTab(controller: widget.controller),
      RibbonTab.review => ReviewTab(controller: widget.controller),
      RibbonTab.view => ViewTab(controller: widget.controller),
    };
  }
}

class _TabStrip extends StatelessWidget {
  const _TabStrip({
    required this.activeTab,
    required this.onTabSelected,
    required this.onShare,
  });

  final RibbonTab activeTab;
  final ValueChanged<RibbonTab> onTabSelected;
  final VoidCallback onShare;

  @override
  Widget build(BuildContext context) {
    return Container(
      height: WordTheme.tabStripHeight,
      color: WordTheme.tabStripSurface,
      child: Row(
        children: [
          Expanded(
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: [
                  const SizedBox(width: WordTheme.trafficLightInset),
                  ...RibbonTab.values.map((tab) => _TabItem(
                        key: Key('ribbon_tab_${tab.name}'),
                        label: tab.label,
                        selected: tab == activeTab,
                        onTap: () => onTabSelected(tab),
                      )),
                ],
              ),
            ),
          ),
          _ShareButton(
            onPressed: onShare,
            tooltip: 'Share with another app',
          ),
          const SizedBox(width: 12),
        ],
      ),
    );
  }
}

class _TabItem extends StatelessWidget {
  const _TabItem({
    super.key,
    required this.label,
    required this.selected,
    required this.onTap,
  });

  final String label;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return RibbonFocusable(
      enabled: true,
      onActivate: onTap,
      builder: (context, {required hovered, required focused}) {
        return Container(
          padding: const EdgeInsets.symmetric(horizontal: 14),
          decoration: BoxDecoration(
            color: (hovered || focused)
                ? WordTheme.ribbonHover
                : Colors.transparent,
            border: Border(
              bottom: BorderSide(
                color: selected
                    ? WordTheme.activeTabUnderline
                    : (focused ? WordTheme.activeTabUnderline.withValues(alpha: 0.5) : Colors.transparent),
                width: selected || focused ? 2 : 0,
              ),
            ),
          ),
          alignment: Alignment.center,
          child: Text(
            label,
            style: selected ? WordTheme.tabLabelActive : WordTheme.tabLabel,
          ),
        );
      },
    );
  }
}

class _ShareButton extends StatelessWidget {
  const _ShareButton({this.onPressed, this.tooltip});

  final VoidCallback? onPressed;
  final String? tooltip;

  @override
  Widget build(BuildContext context) {
    final enabled = onPressed != null;
    final tip = tooltip ?? (enabled ? null : kComingSoonTooltip);

    return wrapRibbonTooltip(
      tip,
      RibbonFocusable(
        key: const Key('ribbon_share'),
        enabled: enabled,
        onActivate: onPressed,
        builder: (context, {required hovered, required focused}) {
          return Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
            decoration: ribbonFocusDecoration(
              fill: (hovered || focused) && enabled
                  ? WordTheme.ribbonHover
                  : Colors.transparent,
              focused: focused,
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  Icons.share_outlined,
                  size: 16,
                  color: enabled ? WordTheme.ribbonText : WordTheme.ribbonTextDisabled,
                ),
                const SizedBox(width: 4),
                Text(
                  'Share',
                  style: WordTheme.tabLabel.copyWith(
                    color: enabled ? WordTheme.ribbonText : WordTheme.ribbonTextDisabled,
                  ),
                ),
              ],
            ),
          );
        },
      ),
    );
  }
}
