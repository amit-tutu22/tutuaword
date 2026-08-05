import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
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
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        _TabStrip(
          activeTab: _activeTab,
          onTabSelected: (tab) => setState(() => _activeTab = tab),
        ),
        Container(
          height: WordTheme.ribbonHeight,
          color: WordTheme.ribbonSurface,
          child: ClipRect(
            child: _buildTabContent(),
          ),
        ),
      ],
    );
  }

  Widget _buildTabContent() {
    return switch (_activeTab) {
      RibbonTab.home => HomeTab(controller: widget.controller),
      RibbonTab.insert => InsertTab(controller: widget.controller),
      RibbonTab.design => const DesignTab(),
      RibbonTab.layout => const LayoutTab(),
      RibbonTab.references => const ReferencesTab(),
      RibbonTab.mailings => const MailingsTab(),
      RibbonTab.review => ReviewTab(controller: widget.controller),
      RibbonTab.view => ViewTab(controller: widget.controller),
    };
  }
}

class _TabStrip extends StatelessWidget {
  const _TabStrip({
    required this.activeTab,
    required this.onTabSelected,
  });

  final RibbonTab activeTab;
  final ValueChanged<RibbonTab> onTabSelected;

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
                        label: tab.label,
                        selected: tab == activeTab,
                        onTap: () => onTabSelected(tab),
                      )),
                ],
              ),
            ),
          ),
          _ShareButton(onPressed: null, tooltip: kComingSoonTooltip),
          const SizedBox(width: 12),
        ],
      ),
    );
  }
}

class _TabItem extends StatefulWidget {
  const _TabItem({
    required this.label,
    required this.selected,
    required this.onTap,
  });

  final String label;
  final bool selected;
  final VoidCallback onTap;

  @override
  State<_TabItem> createState() => _TabItemState();
}

class _TabItemState extends State<_TabItem> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 14),
          decoration: BoxDecoration(
            color: _hovered ? WordTheme.ribbonHover : Colors.transparent,
            border: widget.selected
                ? const Border(
                    bottom: BorderSide(color: WordTheme.activeTabUnderline, width: 2),
                  )
                : null,
          ),
          alignment: Alignment.center,
          child: Text(
            widget.label,
            style: widget.selected ? WordTheme.tabLabelActive : WordTheme.tabLabel,
          ),
        ),
      ),
    );
  }
}

class _ShareButton extends StatefulWidget {
  const _ShareButton({this.onPressed, this.tooltip});

  final VoidCallback? onPressed;
  final String? tooltip;

  @override
  State<_ShareButton> createState() => _ShareButtonState();
}

class _ShareButtonState extends State<_ShareButton> {
  bool _hovered = false;

  @override
  Widget build(BuildContext context) {
    final enabled = widget.onPressed != null;
    final tooltip = widget.tooltip ?? (enabled ? null : kComingSoonTooltip);

    return wrapRibbonTooltip(
      tooltip,
      MouseRegion(
        onEnter: (_) => setState(() => _hovered = true),
        onExit: (_) => setState(() => _hovered = false),
        child: GestureDetector(
          onTap: widget.onPressed,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
            decoration: BoxDecoration(
              color: _hovered && enabled ? WordTheme.ribbonHover : Colors.transparent,
              borderRadius: BorderRadius.circular(3),
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
          ),
        ),
      ),
    );
  }
}
