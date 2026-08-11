import 'package:flutter/material.dart';
import 'package:tutuaword/editor/document_templates.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Category filter for the template gallery (F24.S4).
enum TemplateGalleryCategory {
  all,
  builtin,
  mine,
}

/// Pure filter helpers for the gallery (unit-tested).
abstract final class TemplateGalleryFilter {
  static bool matchesQuery(String haystack, String query) {
    final q = query.trim().toLowerCase();
    if (q.isEmpty) return true;
    return haystack.toLowerCase().contains(q);
  }

  static List<DocumentTemplateSpec> filterBuiltins({
    required String query,
    required TemplateGalleryCategory category,
  }) {
    if (category == TemplateGalleryCategory.mine) return const [];
    return DocumentTemplateSpec.all
        .where(
          (t) =>
              matchesQuery(t.title, query) || matchesQuery(t.subtitle, query),
        )
        .toList();
  }

  static List<UserTemplateEntry> filterUser({
    required List<UserTemplateEntry> userTemplates,
    required String query,
    required TemplateGalleryCategory category,
  }) {
    if (category == TemplateGalleryCategory.builtin) return const [];
    return userTemplates
        .where(
          (e) =>
              matchesQuery(e.title, query) ||
              matchesQuery(e.themeName, query),
        )
        .toList();
  }
}

/// Accent color for theme-bound template preview cards.
Color templateThemeAccent(String themeName) {
  switch (themeName) {
    case 'Facet':
      return const Color(0xFF217346);
    case 'Ion':
      return const Color(0xFFC45911);
    case 'Office':
    default:
      return WordTheme.activeTabUnderline;
  }
}

/// Word-style card gallery for New from Template (F24.S4).
class NewFromTemplateDialog extends StatefulWidget {
  const NewFromTemplateDialog({
    super.key,
    this.userTemplates = const [],
  });

  final List<UserTemplateEntry> userTemplates;

  static Future<TemplateSelection?> show(
    BuildContext context, {
    List<UserTemplateEntry> userTemplates = const [],
  }) {
    return showDialog<TemplateSelection>(
      context: context,
      barrierDismissible: true,
      builder: (context) => NewFromTemplateDialog(userTemplates: userTemplates),
    );
  }

  @override
  State<NewFromTemplateDialog> createState() => _NewFromTemplateDialogState();
}

class _NewFromTemplateDialogState extends State<NewFromTemplateDialog> {
  final _searchController = TextEditingController();
  TemplateGalleryCategory _category = TemplateGalleryCategory.all;
  TemplateSelection? _selected;

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  void _select(TemplateSelection selection) {
    setState(() => _selected = selection);
  }

  void _create([TemplateSelection? selection]) {
    final choice = selection ?? _selected;
    if (choice == null) return;
    Navigator.of(context).pop(choice);
  }

  @override
  Widget build(BuildContext context) {
    final query = _searchController.text;
    final builtins = TemplateGalleryFilter.filterBuiltins(
      query: query,
      category: _category,
    );
    final mine = TemplateGalleryFilter.filterUser(
      userTemplates: widget.userTemplates,
      query: query,
      category: _category,
    );
    final showMine = mine.isNotEmpty;
    final showBuiltin = builtins.isNotEmpty;

    return AlertDialog(
      key: const Key('new_from_template_dialog'),
      title: const Text('New from Template'),
      content: SizedBox(
        width: 640,
        height: 460,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            TextField(
              key: const Key('template_gallery_search'),
              controller: _searchController,
              decoration: const InputDecoration(
                isDense: true,
                prefixIcon: Icon(Icons.search, size: 18),
                hintText: 'Search templates',
                border: OutlineInputBorder(),
              ),
              onChanged: (_) => setState(() {}),
            ),
            const SizedBox(height: 10),
            Wrap(
              spacing: 8,
              children: [
                for (final entry in const [
                  (TemplateGalleryCategory.all, 'All', 'template_gallery_cat_all'),
                  (
                    TemplateGalleryCategory.builtin,
                    'Built-in',
                    'template_gallery_cat_builtin'
                  ),
                  (
                    TemplateGalleryCategory.mine,
                    'My Templates',
                    'template_gallery_cat_mine'
                  ),
                ])
                  ChoiceChip(
                    key: Key(entry.$3),
                    label: Text(entry.$2),
                    selected: _category == entry.$1,
                    onSelected: (_) => setState(() => _category = entry.$1),
                  ),
              ],
            ),
            const SizedBox(height: 12),
            Expanded(
              child: SingleChildScrollView(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    if (showMine) ...[
                      const Text(
                        'My Templates',
                        key: Key('my_templates_section'),
                        style: TextStyle(
                          fontSize: 12,
                          fontWeight: FontWeight.w600,
                          color: WordTheme.ribbonText,
                        ),
                      ),
                      const SizedBox(height: 8),
                      Wrap(
                        spacing: 10,
                        runSpacing: 10,
                        children: [
                          for (final entry in mine)
                            _TemplateGalleryCard(
                              key: Key('user_template_${entry.id}'),
                              title: entry.title,
                              subtitle: 'Theme: ${entry.themeName}',
                              themeName: entry.themeName,
                              previewKind: TemplatePreviewKind.user,
                              selected: _selected?.user?.id == entry.id,
                              onTap: () =>
                                  _select(TemplateSelection.user(entry)),
                              onActivate: () =>
                                  _create(TemplateSelection.user(entry)),
                            ),
                        ],
                      ),
                      if (showBuiltin) const SizedBox(height: 16),
                    ],
                    if (showBuiltin) ...[
                      const Text(
                        'Built-in',
                        key: Key('builtin_templates_section'),
                        style: TextStyle(
                          fontSize: 12,
                          fontWeight: FontWeight.w600,
                          color: WordTheme.ribbonText,
                        ),
                      ),
                      const SizedBox(height: 8),
                      Wrap(
                        spacing: 10,
                        runSpacing: 10,
                        children: [
                          for (final template in builtins)
                            _TemplateGalleryCard(
                              key: Key('template_${template.id}'),
                              title: template.title,
                              subtitle: template.subtitle,
                              themeName: template.themeName,
                              previewKind:
                                  TemplatePreviewKind.forBuiltin(template.id),
                              selected: _selected?.spec?.id == template.id,
                              onTap: () =>
                                  _select(TemplateSelection.builtin(template)),
                              onActivate: () => _create(
                                TemplateSelection.builtin(template),
                              ),
                            ),
                        ],
                      ),
                    ],
                    if (!showMine && !showBuiltin)
                      const Padding(
                        padding: EdgeInsets.only(top: 40),
                        child: Center(
                          child: Text(
                            'No templates match',
                            key: Key('template_gallery_empty'),
                            style: WordTheme.ribbonGroupLabel,
                          ),
                        ),
                      ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 12),
            if (_selected != null)
              Text(
                key: const Key('template_gallery_selection'),
                '${_selected!.title} · ${_selected!.themeName}',
                style: WordTheme.ribbonGroupLabel,
              ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('new_from_template_cancel'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        TextButton(
          key: const Key('template_gallery_create'),
          onPressed: _selected == null ? null : () => _create(),
          child: const Text('Create'),
        ),
      ],
    );
  }
}

/// Stylized page preview layout for gallery cards.
enum TemplatePreviewKind {
  resume,
  letter,
  invoice,
  brochure,
  newsletter,
  proposal,
  research,
  user;

  static TemplatePreviewKind forBuiltin(String id) {
    switch (id) {
      case 'resume':
        return resume;
      case 'letter':
        return letter;
      case 'invoice':
        return invoice;
      case 'brochure':
        return brochure;
      case 'newsletter':
        return newsletter;
      case 'business_proposal':
        return proposal;
      case 'research_paper':
        return research;
      default:
        return user;
    }
  }
}

class _TemplateGalleryCard extends StatelessWidget {
  const _TemplateGalleryCard({
    super.key,
    required this.title,
    required this.subtitle,
    required this.themeName,
    required this.previewKind,
    required this.selected,
    required this.onTap,
    required this.onActivate,
  });

  final String title;
  final String subtitle;
  final String themeName;
  final TemplatePreviewKind previewKind;
  final bool selected;
  final VoidCallback onTap;
  /// Second click on an already-selected card creates the document.
  final VoidCallback onActivate;

  @override
  Widget build(BuildContext context) {
    final accent = templateThemeAccent(themeName);
    final border = selected
        ? WordTheme.activeTabUnderline
        : const Color(0xFFB0B0B0);

    return Tooltip(
      message: subtitle,
      child: OutlinedButton(
        onPressed: selected ? onActivate : onTap,
        style: OutlinedButton.styleFrom(
          padding: const EdgeInsets.fromLTRB(8, 8, 8, 8),
          backgroundColor: selected ? const Color(0xFFE8F0F8) : Colors.white,
          side: BorderSide(
            color: border,
            width: selected ? 1.5 : 1,
          ),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(3),
          ),
          fixedSize: const Size(132, 204),
          alignment: Alignment.topLeft,
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Container(
              width: 116,
              height: 148,
              decoration: BoxDecoration(
                color: Colors.white,
                border: Border.all(color: const Color(0xFFD0D0D0)),
                boxShadow: const [
                  BoxShadow(
                    color: Color(0x14000000),
                    blurRadius: 2,
                    offset: Offset(0, 1),
                  ),
                ],
              ),
              child: CustomPaint(
                painter: _TemplatePreviewPainter(
                  kind: previewKind,
                  accent: accent,
                ),
              ),
            ),
            const SizedBox(height: 8),
            Text(
              title,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.w600,
                color: WordTheme.ribbonText,
              ),
            ),
            const SizedBox(height: 2),
            Text(
              themeName,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: WordTheme.ribbonGroupLabel,
            ),
          ],
        ),
      ),
    );
  }
}

class _TemplatePreviewPainter extends CustomPainter {
  _TemplatePreviewPainter({required this.kind, required this.accent});

  final TemplatePreviewKind kind;
  final Color accent;

  @override
  void paint(Canvas canvas, Size size) {
    final line = Paint()
      ..color = const Color(0xFFCCCCCC)
      ..strokeWidth = 1;
    final accentPaint = Paint()..color = accent;
    final soft = Paint()..color = accent.withValues(alpha: 0.18);

    canvas.drawRect(Offset.zero & size, Paint()..color = Colors.white);
    canvas.drawRect(Rect.fromLTWH(0, 0, size.width, 10), accentPaint);

    switch (kind) {
      case TemplatePreviewKind.resume:
        canvas.drawRect(Rect.fromLTWH(8, 18, 28, 28), soft);
        _lines(canvas, line, 44, 22, size.width - 12, 3, 5);
        _lines(canvas, line, 8, 56, size.width - 12, 4, 7);
        break;
      case TemplatePreviewKind.letter:
        _lines(canvas, line, 8, 20, 50, 2, 4);
        _lines(canvas, line, 8, 48, size.width - 12, 5, 6);
        break;
      case TemplatePreviewKind.invoice:
        canvas.drawRect(Rect.fromLTWH(8, 18, size.width - 16, 14), soft);
        for (var r = 0; r < 4; r++) {
          final y = 40.0 + r * 14;
          canvas.drawRect(Rect.fromLTWH(8, y, size.width - 16, 12), line);
        }
        break;
      case TemplatePreviewKind.brochure:
        canvas.drawRect(Rect.fromLTWH(8, 18, size.width - 16, 36), soft);
        _lines(canvas, line, 8, 62, size.width - 12, 4, 5);
        break;
      case TemplatePreviewKind.newsletter:
        canvas.drawRect(Rect.fromLTWH(8, 18, size.width - 16, 18), accentPaint);
        _lines(canvas, line, 8, 44, (size.width / 2) - 10, 4, 5);
        _lines(canvas, line, size.width / 2 + 2, 44, size.width - 10, 4, 5);
        break;
      case TemplatePreviewKind.proposal:
        canvas.drawRect(Rect.fromLTWH(8, 18, size.width - 16, 10), soft);
        _lines(canvas, line, 8, 36, size.width - 12, 6, 6);
        break;
      case TemplatePreviewKind.research:
        _lines(canvas, line, 20, 20, size.width - 20, 1, 4);
        _lines(canvas, line, 8, 36, size.width - 12, 7, 5);
        break;
      case TemplatePreviewKind.user:
        canvas.drawRect(Rect.fromLTWH(8, 18, size.width - 16, 8), soft);
        _lines(canvas, line, 8, 36, size.width - 12, 6, 6);
        break;
    }
  }

  void _lines(
    Canvas canvas,
    Paint paint,
    double x0,
    double y0,
    double x1,
    int count,
    double gap,
  ) {
    for (var i = 0; i < count; i++) {
      final y = y0 + i * gap;
      canvas.drawLine(Offset(x0, y), Offset(x1, y), paint);
    }
  }

  @override
  bool shouldRepaint(covariant _TemplatePreviewPainter oldDelegate) {
    return oldDelegate.kind != kind || oldDelegate.accent != accent;
  }
}
