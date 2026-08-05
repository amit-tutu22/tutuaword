import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';
import 'package:tutuaword/ui/word_theme.dart';

class HomeTab extends StatelessWidget {
  const HomeTab({super.key, required this.controller});

  final EditorController controller;

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
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
                      onPressed: () {},
                    ),
                    const SizedBox(width: 4),
                    RibbonDropdown(
                      value: controller.fontSize.toInt().toString(),
                      width: 40,
                      onPressed: () {},
                    ),
                    const SizedBox(width: 4),
                    RibbonIconButton(icon: Icons.text_increase, onPressed: controller.increaseFontSize),
                    RibbonIconButton(icon: Icons.text_decrease, onPressed: controller.decreaseFontSize),
                    RibbonIconButton(icon: Icons.change_circle_outlined, onPressed: null),
                    RibbonIconButton(icon: Icons.format_clear, onPressed: null),
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
                    RibbonIconButton(icon: Icons.format_color_text, onPressed: null),
                    RibbonIconButton(icon: Icons.format_color_fill, onPressed: null),
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
                    RibbonIconButton(icon: Icons.format_list_numbered, onPressed: null),
                    RibbonIconButton(icon: Icons.format_indent_decrease, onPressed: null),
                    RibbonIconButton(icon: Icons.format_indent_increase, onPressed: null),
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
                  selected: true,
                  onPressed: () {},
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
      ),
    );
  }
}
