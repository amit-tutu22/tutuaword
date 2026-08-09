import 'package:flutter/material.dart';
import 'package:tutuaword/editor/recent_symbols.dart';
import 'package:tutuaword/editor/symbol_catalog.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Word-like Insert Symbol dialog with category grid (F15.S1) and recents (F15.S3).
class SymbolDialog extends StatefulWidget {
  const SymbolDialog({super.key, required this.recentStore});

  final RecentSymbolsStore recentStore;

  static Future<String?> show(
    BuildContext context, {
    required RecentSymbolsStore recentStore,
  }) {
    return showDialog<String>(
      context: context,
      barrierDismissible: true,
      builder: (context) => SymbolDialog(recentStore: recentStore),
    );
  }

  @override
  State<SymbolDialog> createState() => _SymbolDialogState();
}

class _SymbolDialogState extends State<SymbolDialog> {
  late String _categoryId;
  SymbolEntry? _hovered;

  @override
  void initState() {
    super.initState();
    final recents = widget.recentStore.entries();
    _categoryId = recents.isNotEmpty
        ? RecentSymbolsStore.recentCategoryId
        : SymbolCatalog.categories.first.id;
  }

  bool get _isRecent => _categoryId == RecentSymbolsStore.recentCategoryId;

  SymbolCategory get _category => SymbolCatalog.categories.firstWhere(
        (c) => c.id == _categoryId,
      );

  bool get _isEmoji => SymbolCatalog.isEmojiCategory(_categoryId);

  List<SymbolEntry> get _gridSymbols =>
      _isRecent ? widget.recentStore.entries() : _category.symbols;

  void _insert(SymbolEntry entry) {
    Navigator.pop(context, entry.character);
  }

  @override
  Widget build(BuildContext context) {
    final preview = _hovered;
    final recents = widget.recentStore.entries();
    return AlertDialog(
      key: const Key('symbol_dialog'),
      title: const Text('Symbol'),
      content: SizedBox(
        width: 420,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
            if (recents.isNotEmpty) ...[
              Text('Recently used', style: WordTheme.ribbonLabel),
              const SizedBox(height: 6),
              SizedBox(
                height: 44,
                child: ListView.separated(
                  key: const Key('symbol_recent_row'),
                  scrollDirection: Axis.horizontal,
                  itemCount: recents.length,
                  separatorBuilder: (_, __) => const SizedBox(width: 4),
                  itemBuilder: (context, index) {
                    final entry = recents[index];
                    return Material(
                      color: WordTheme.ribbonSurface,
                      child: InkWell(
                        key: Key('symbol_recent_${entry.id}'),
                        onTap: () => _insert(entry),
                        onHover: (hover) {
                          setState(() => _hovered = hover ? entry : null);
                        },
                        child: Padding(
                          padding: const EdgeInsets.symmetric(horizontal: 10),
                          child: Center(
                            child: Text(
                              entry.character,
                              style: const TextStyle(fontSize: 22),
                            ),
                          ),
                        ),
                      ),
                    );
                  },
                ),
              ),
              const SizedBox(height: 12),
            ],
            DropdownButtonFormField<String>(
              key: const Key('symbol_category_dropdown'),
              value: _categoryId,
              decoration: const InputDecoration(
                labelText: 'Subset',
                isDense: true,
              ),
              items: [
                if (recents.isNotEmpty)
                  const DropdownMenuItem(
                    value: RecentSymbolsStore.recentCategoryId,
                    child: Text('Recently Used'),
                  ),
                for (final category in SymbolCatalog.categories)
                  DropdownMenuItem(
                    value: category.id,
                    child: Text(category.name),
                  ),
              ],
              onChanged: (value) {
                if (value == null) return;
                setState(() {
                  _categoryId = value;
                  _hovered = null;
                });
              },
            ),
            const SizedBox(height: 12),
            SizedBox(
              height: _isEmoji ? 260 : 220,
              child: GridView.builder(
                key: Key('symbol_grid_$_categoryId'),
                gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                  crossAxisCount: _isEmoji ? 6 : 8,
                  mainAxisSpacing: 4,
                  crossAxisSpacing: 4,
                ),
                itemCount: _gridSymbols.length,
                itemBuilder: (context, index) {
                  final entry = _gridSymbols[index];
                  return Material(
                    color: WordTheme.ribbonSurface,
                    child: InkWell(
                      key: Key('symbol_cell_${entry.id}'),
                      onTap: () => _insert(entry),
                      onHover: (hover) {
                        setState(() => _hovered = hover ? entry : null);
                      },
                      child: Tooltip(
                        message: entry.description ?? entry.character,
                        child: Center(
                          child: Text(
                            entry.character,
                            style: TextStyle(fontSize: _isEmoji ? 26 : 20),
                          ),
                        ),
                      ),
                    ),
                  );
                },
              ),
            ),
            const SizedBox(height: 12),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: WordTheme.ribbonSurface,
                borderRadius: BorderRadius.circular(4),
                border: Border.all(color: WordTheme.groupDivider),
              ),
              child: Text(
                preview == null
                    ? 'Click a symbol to insert'
                    : '${preview.description ?? preview.id}: ${preview.character}',
                key: const Key('symbol_preview'),
                style: WordTheme.ribbonLabel,
              ),
            ),
          ],
          ),
        ),
      ),
      actions: [
        TextButton(
          key: const Key('symbol_cancel'),
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
      ],
    );
  }
}
