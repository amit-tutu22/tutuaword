import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/page_setup.dart';
import 'package:tutuaword/ui/ribbon_color_picker.dart';
import 'package:tutuaword/ui/ribbon_widgets.dart';

class DesignTab extends StatelessWidget {
  const DesignTab({super.key, required this.controller});

  final EditorController controller;

  static const _galleryThemes = <(String, TextStyle)>[
    ('Office', TextStyle(fontSize: 9)),
    ('Facet', TextStyle(fontSize: 9, fontWeight: FontWeight.w600)),
    ('Ion', TextStyle(fontSize: 9)),
  ];

  Future<void> _openWatermarkMenu(BuildContext context) async {
    final box = context.findRenderObject() as RenderBox?;
    if (box == null) return;
    final items = [
      ...PageSetupPresets.watermarkPresets,
      if (controller.watermarkText != null) PageSetupPresets.removeWatermarkLabel,
    ];
    await showRibbonPresetMenu(context, box, items, (value) {
      if (value == PageSetupPresets.removeWatermarkLabel) {
        controller.clearWatermark();
      } else {
        controller.applyWatermark(value);
      }
    }, minWidth: 140, maxWidth: 220);
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        return RibbonTabScroller(
          children: [
            RibbonGroup(
              label: 'Document Formatting',
              child: Row(
                children: [
                  for (final entry in _galleryThemes)
                    StyleGalleryCard(
                      label: entry.$1,
                      previewStyle: entry.$2,
                      selected: controller.documentThemeName == entry.$1,
                      onPressed: () => controller.applyDocumentTheme(entry.$1),
                    ),
                  RibbonIconButton(icon: Icons.chevron_right, onPressed: null),
                ],
              ),
            ),
            RibbonGroup(
              label: 'Page Background',
              showDivider: false,
              child: Row(
                children: [
                  Builder(
                    builder: (context) => RibbonLargeButton(
                      icon: Icons.water_drop_outlined,
                      label: 'Watermark',
                      tooltip: controller.watermarkText ?? 'Watermark',
                      onPressed: () => _openWatermarkMenu(context),
                    ),
                  ),
                  RibbonColorButton(
                    icon: Icons.palette_outlined,
                    tooltip: 'Page Color',
                    barColor: controller.pageColor ?? const Color(0xFFFFFFFF),
                    onColorSelected: (selection) =>
                        controller.applyPageColor(selection.color),
                    onClear: () => controller.applyPageColor(null),
                  ),
                  RibbonLargeButton(
                    icon: Icons.border_style,
                    label: 'Page\nBorders',
                    onPressed: null,
                  ),
                ],
              ),
            ),
          ],
        );
      },
    );
  }
}
