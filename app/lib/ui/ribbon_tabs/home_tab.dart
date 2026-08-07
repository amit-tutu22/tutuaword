import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_color_picker.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';
import 'package:tutuaword/ui/word_theme.dart';

class HomeTab extends StatelessWidget {
  const HomeTab({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        return RibbonTabScroller(
          children: [
              RibbonGroup(
                label: 'Clipboard',
            child: Row(
              children: [
                RibbonLargeButton(
                  icon: Icons.content_paste,
                  label: 'Paste',
                  onPressed: () => controller.paste(),
                ),
                const SizedBox(width: 4),
                Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    RibbonIconButton(
                      icon: Icons.content_cut,
                      tooltip: 'Cut',
                      onPressed: controller.cutSelection,
                      iconSize: 14,
                    ),
                    RibbonIconButton(
                      icon: Icons.content_copy,
                      tooltip: 'Copy',
                      onPressed: controller.copySelection,
                      iconSize: 14,
                    ),
                    RibbonIconButton(
                      icon: Icons.format_paint,
                      tooltip: 'Format Painter',
                      onPressed: null,
                      iconSize: 14,
                    ),
                  ],
                ),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Font',
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Row(
                  children: [
                    RibbonDropdown(
                      value: controller.fontFamily,
                      width: 120,
                      items: kRibbonFontFamilies,
                      onSelected: controller.setFontFamily,
                      itemStyle: (family) => TextStyle(fontFamily: family, fontSize: 11),
                    ),
                    const SizedBox(width: 4),
                    RibbonDropdown(
                      value: formatRibbonFontSize(controller.fontSize),
                      width: 48,
                      items: kRibbonFontSizes,
                      onSelected: (size) => controller.setFontSize(double.parse(size)),
                    ),
                    const SizedBox(width: 4),
                    RibbonIconButton(icon: Icons.text_increase, onPressed: controller.increaseFontSize),
                    RibbonIconButton(icon: Icons.text_decrease, onPressed: controller.decreaseFontSize),
                    RibbonIconButton(icon: Icons.change_circle_outlined, onPressed: null),
                    RibbonIconButton(
                      icon: Icons.format_clear,
                      tooltip: 'Clear Formatting',
                      onPressed: controller.clearFormatting,
                    ),
                  ],
                ),
                const SizedBox(height: 2),
                Row(
                  children: [
                    RibbonToggleButton(
                      icon: Icons.format_bold,
                      tooltip: 'Bold',
                      selected: controller.bold,
                      onPressed: controller.toggleBold,
                    ),
                    RibbonToggleButton(
                      icon: Icons.format_italic,
                      tooltip: 'Italic',
                      selected: controller.italic,
                      onPressed: controller.toggleItalic,
                    ),
                    RibbonToggleButton(
                      icon: Icons.format_underline,
                      tooltip: 'Underline',
                      selected: controller.underline,
                      onPressed: controller.toggleUnderline,
                    ),
                    RibbonToggleButton(
                      icon: Icons.format_strikethrough,
                      tooltip: 'Strikethrough',
                      selected: controller.strikethrough,
                      onPressed: controller.toggleStrikethrough,
                    ),
                    RibbonToggleButton(
                      icon: Icons.subscript,
                      tooltip: 'Subscript',
                      selected: controller.subscript,
                      onPressed: controller.toggleSubscript,
                    ),
                    RibbonToggleButton(
                      icon: Icons.superscript,
                      tooltip: 'Superscript',
                      selected: controller.superscript,
                      onPressed: controller.toggleSuperscript,
                    ),
                    RibbonColorButton(
                      icon: Icons.format_color_text,
                      tooltip: 'Font Color',
                      barColor: controller.fontColor,
                      onColorSelected: controller.setFontColor,
                      onAutomatic: () => controller.setFontColor(Colors.black),
                    ),
                    RibbonColorButton(
                      icon: Icons.format_color_fill,
                      tooltip: 'Text Highlight Color',
                      barColor: controller.highlightColor ?? const Color(0xFFFFFF00),
                      onColorSelected: controller.setHighlight,
                      onClear: controller.clearHighlight,
                    ),
                    RibbonTextToggleButton(
                      text: 'AA',
                      tooltip: 'All Caps',
                      selected: controller.allCaps,
                      onPressed: controller.toggleAllCaps,
                    ),
                    RibbonTextToggleButton(
                      text: 'Aa',
                      tooltip: 'Small Caps',
                      selected: controller.smallCaps,
                      onPressed: controller.toggleSmallCaps,
                      textStyle: const TextStyle(
                        fontSize: 11,
                        fontFeatures: [FontFeature.enable('smcp')],
                      ),
                    ),
                    RibbonToggleButton(
                      icon: Icons.visibility_off,
                      tooltip: 'Hidden',
                      selected: controller.hidden,
                      onPressed: controller.toggleHidden,
                    ),
                    RibbonToggleButton(
                      icon: Icons.link,
                      tooltip: 'Ligatures',
                      selected: controller.ligatures,
                      onPressed: controller.toggleLigatures,
                    ),
                  ],
                ),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Paragraph',
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Row(
                  children: [
                    RibbonIconButton(
                      icon: Icons.format_list_bulleted,
                      tooltip: 'Bullets',
                      onPressed: controller.applyBulletList,
                    ),
                    RibbonIconButton(
                      icon: Icons.format_list_numbered,
                      tooltip: 'Numbering',
                      onPressed: controller.applyNumberedList,
                    ),
                    RibbonIconButton(
                      icon: Icons.format_indent_decrease,
                      tooltip: 'Decrease Indent',
                      onPressed: controller.decreaseIndent,
                    ),
                    RibbonIconButton(
                      icon: Icons.format_indent_increase,
                      tooltip: 'Increase Indent',
                      onPressed: controller.increaseIndent,
                    ),
                    RibbonIconButton(icon: Icons.sort, onPressed: null),
                    RibbonIconButton(icon: Icons.visibility, onPressed: null),
                  ],
                ),
                const SizedBox(height: 2),
                Row(
                  children: [
                    RibbonToggleButton(
                      icon: Icons.format_align_left,
                      tooltip: 'Align Left',
                      selected: controller.alignment == TextAlign.left,
                      onPressed: () => controller.setAlignment(TextAlign.left),
                    ),
                    RibbonToggleButton(
                      icon: Icons.format_align_center,
                      tooltip: 'Center',
                      selected: controller.alignment == TextAlign.center,
                      onPressed: () => controller.setAlignment(TextAlign.center),
                    ),
                    RibbonToggleButton(
                      icon: Icons.format_align_right,
                      tooltip: 'Align Right',
                      selected: controller.alignment == TextAlign.right,
                      onPressed: () => controller.setAlignment(TextAlign.right),
                    ),
                    RibbonToggleButton(
                      icon: Icons.format_align_justify,
                      tooltip: 'Justify',
                      selected: controller.alignment == TextAlign.justify,
                      onPressed: () => controller.setAlignment(TextAlign.justify),
                    ),
                    RibbonIconButton(icon: Icons.format_line_spacing, onPressed: null),
                    RibbonIconButton(icon: Icons.border_all, onPressed: null),
                  ],
                ),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Styles',
            child: Row(
              children: [
                StyleGalleryCard(
                  label: 'Normal',
                  previewStyle: const TextStyle(fontSize: 9),
                  selected: controller.activeParagraphStyle == 'Normal',
                  onPressed: controller.applyNormalStyle,
                ),
                StyleGalleryCard(
                  label: 'No Spacing',
                  previewStyle: const TextStyle(fontSize: 9, height: 1.0),
                  onPressed: null,
                ),
                StyleGalleryCard(
                  label: 'Heading 1',
                  previewStyle: const TextStyle(fontSize: 11, fontWeight: FontWeight.bold),
                  onPressed: controller.applyHeading1,
                ),
                RibbonIconButton(icon: Icons.chevron_right, onPressed: null),
                RibbonTextButton(label: 'Styles\nPane', onPressed: null),
              ],
            ),
          ),
          RibbonGroup(
            label: 'Add-ins',
            showDivider: false,
            child: RibbonTextButton(
              label: 'Add-ins',
              icon: Icons.extension_outlined,
              onPressed: null,
            ),
          ),
          ],
        );
      },
    );
  }
}
