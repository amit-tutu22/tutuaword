import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:tutuaword/bridge/bookmark_entry.dart';
import 'package:tutuaword/bridge/outline_entry.dart';
import 'package:tutuaword/ui/word_theme.dart';

enum GoToTargetKind { page, bookmark, heading }

/// Result of a successful Go To action (F19.S4).
sealed class GoToResult {
  const GoToResult();
}

class GoToPageResult extends GoToResult {
  const GoToPageResult(this.pageIndex);
  /// Zero-based page index.
  final int pageIndex;
}

class GoToBookmarkResult extends GoToResult {
  const GoToBookmarkResult(this.entry);
  final DocumentBookmarkEntry entry;
}

class GoToHeadingResult extends GoToResult {
  const GoToHeadingResult(this.entry);
  final DocumentOutlineEntry entry;
}

/// Word-like Go To dialog: page, bookmark, or heading.
class GoToDialog extends StatefulWidget {
  const GoToDialog({
    super.key,
    required this.pageCount,
    required this.currentPage,
    required this.bookmarks,
    required this.headings,
  });

  final int pageCount;
  final int currentPage;
  final List<DocumentBookmarkEntry> bookmarks;
  final List<DocumentOutlineEntry> headings;

  static Future<GoToResult?> show(
    BuildContext context, {
    required int pageCount,
    required int currentPage,
    required List<DocumentBookmarkEntry> bookmarks,
    required List<DocumentOutlineEntry> headings,
  }) {
    return showDialog<GoToResult>(
      context: context,
      barrierDismissible: true,
      builder: (context) => GoToDialog(
        pageCount: pageCount,
        currentPage: currentPage,
        bookmarks: bookmarks,
        headings: headings,
      ),
    );
  }

  @override
  State<GoToDialog> createState() => _GoToDialogState();
}

class _GoToDialogState extends State<GoToDialog> {
  GoToTargetKind _kind = GoToTargetKind.page;
  late final TextEditingController _pageController;
  DocumentBookmarkEntry? _selectedBookmark;
  DocumentOutlineEntry? _selectedHeading;

  @override
  void initState() {
    super.initState();
    final displayPage = (widget.currentPage + 1).clamp(1, widget.pageCount.clamp(1, 999999));
    _pageController = TextEditingController(text: '$displayPage');
    if (widget.bookmarks.isNotEmpty) {
      _selectedBookmark = widget.bookmarks.first;
    }
    if (widget.headings.isNotEmpty) {
      _selectedHeading = widget.headings.first;
    }
  }

  @override
  void dispose() {
    _pageController.dispose();
    super.dispose();
  }

  void _goTo() {
    switch (_kind) {
      case GoToTargetKind.page:
        final parsed = int.tryParse(_pageController.text.trim());
        if (parsed == null || widget.pageCount <= 0) return;
        final index = (parsed - 1).clamp(0, widget.pageCount - 1);
        Navigator.pop(context, GoToPageResult(index));
      case GoToTargetKind.bookmark:
        final entry = _selectedBookmark;
        if (entry == null) return;
        Navigator.pop(context, GoToBookmarkResult(entry));
      case GoToTargetKind.heading:
        final entry = _selectedHeading;
        if (entry == null) return;
        Navigator.pop(context, GoToHeadingResult(entry));
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('goto_dialog'),
      title: const Text('Go To'),
      content: SizedBox(
        width: 420,
        height: 280,
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            SizedBox(
              width: 110,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Text('Go to what:', style: WordTheme.ribbonLabel),
                  const SizedBox(height: 8),
                  _TargetTile(
                    key: const Key('goto_target_page'),
                    label: 'Page',
                    selected: _kind == GoToTargetKind.page,
                    onTap: () => setState(() => _kind = GoToTargetKind.page),
                  ),
                  _TargetTile(
                    key: const Key('goto_target_bookmark'),
                    label: 'Bookmark',
                    selected: _kind == GoToTargetKind.bookmark,
                    onTap: () => setState(() => _kind = GoToTargetKind.bookmark),
                  ),
                  _TargetTile(
                    key: const Key('goto_target_heading'),
                    label: 'Heading',
                    selected: _kind == GoToTargetKind.heading,
                    onTap: () => setState(() => _kind = GoToTargetKind.heading),
                  ),
                ],
              ),
            ),
            const VerticalDivider(width: 24),
            Expanded(child: _buildTargetPane()),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
        TextButton(
          key: const Key('goto_go_button'),
          onPressed: _goTo,
          child: const Text('Go To'),
        ),
      ],
    );
  }

  Widget _buildTargetPane() {
    switch (_kind) {
      case GoToTargetKind.page:
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Enter page number (1–${widget.pageCount.clamp(1, 999999)}):',
              style: WordTheme.ribbonLabel,
            ),
            const SizedBox(height: 8),
            TextField(
              key: const Key('goto_page_field'),
              controller: _pageController,
              autofocus: true,
              keyboardType: TextInputType.number,
              inputFormatters: [FilteringTextInputFormatter.digitsOnly],
              decoration: const InputDecoration(
                labelText: 'Page number',
                border: OutlineInputBorder(),
                isDense: true,
              ),
              onSubmitted: (_) => _goTo(),
            ),
          ],
        );
      case GoToTargetKind.bookmark:
        if (widget.bookmarks.isEmpty) {
          return Text('No bookmarks in this document.', style: WordTheme.ribbonLabel);
        }
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Select a bookmark:', style: WordTheme.ribbonLabel),
            const SizedBox(height: 8),
            Expanded(
              child: ListView.builder(
                key: const Key('goto_bookmark_list'),
                itemCount: widget.bookmarks.length,
                itemBuilder: (context, index) {
                  final entry = widget.bookmarks[index];
                  final selected = identical(_selectedBookmark, entry) ||
                      _selectedBookmark?.name == entry.name;
                  return ListTile(
                    key: Key('goto_bookmark_${entry.name}'),
                    dense: true,
                    selected: selected,
                    title: Text(entry.name),
                    subtitle: Text('Page ${entry.page + 1}'),
                    onTap: () => setState(() => _selectedBookmark = entry),
                  );
                },
              ),
            ),
          ],
        );
      case GoToTargetKind.heading:
        if (widget.headings.isEmpty) {
          return Text('No headings in this document.', style: WordTheme.ribbonLabel);
        }
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Select a heading:', style: WordTheme.ribbonLabel),
            const SizedBox(height: 8),
            Expanded(
              child: ListView.builder(
                key: const Key('goto_heading_list'),
                itemCount: widget.headings.length,
                itemBuilder: (context, index) {
                  final entry = widget.headings[index];
                  final selected = identical(_selectedHeading, entry) ||
                      _selectedHeading?.paragraphId == entry.paragraphId;
                  return ListTile(
                    key: Key('goto_heading_${entry.paragraphId}'),
                    dense: true,
                    selected: selected,
                    title: Padding(
                      padding: EdgeInsets.only(left: entry.level * 12.0),
                      child: Text(entry.text),
                    ),
                    subtitle: Text('Page ${entry.page + 1}'),
                    onTap: () => setState(() => _selectedHeading = entry),
                  );
                },
              ),
            ),
          ],
        );
    }
  }
}

class _TargetTile extends StatelessWidget {
  const _TargetTile({
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
    return Material(
      color: selected ? WordTheme.ribbonSelected : Colors.transparent,
      child: InkWell(
        onTap: onTap,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 10),
          child: Text(
            label,
            style: WordTheme.ribbonLabel.copyWith(
              fontWeight: selected ? FontWeight.w600 : FontWeight.w400,
            ),
          ),
        ),
      ),
    );
  }
}
