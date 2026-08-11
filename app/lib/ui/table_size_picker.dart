import 'package:flutter/material.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Chosen row/column counts from [TableSizePicker.show].
typedef TableSize = ({int rows, int cols});

/// Word-style Insert Table size grid (hover to preview, click to confirm).
class TableSizePicker extends StatefulWidget {
  const TableSizePicker({
    super.key,
    this.maxRows = 8,
    this.maxCols = 10,
    required this.onSelected,
  });

  final int maxRows;
  final int maxCols;
  final ValueChanged<TableSize> onSelected;

  /// Shows an anchored popup under [anchorContext] and returns the chosen size,
  /// or `null` if dismissed.
  static Future<TableSize?> show(BuildContext anchorContext) async {
    final box = anchorContext.findRenderObject() as RenderBox?;
    if (box == null) return null;
    final overlay = Overlay.of(anchorContext).context.findRenderObject() as RenderBox;
    final origin = box.localToGlobal(Offset.zero, ancestor: overlay);
    final size = box.size;

    return showGeneralDialog<TableSize>(
      context: anchorContext,
      barrierDismissible: true,
      barrierLabel: 'Dismiss',
      barrierColor: Colors.transparent,
      pageBuilder: (context, animation, secondaryAnimation) {
        return Stack(
          children: [
            Positioned(
              left: origin.dx.clamp(8.0, overlay.size.width - 240),
              top: origin.dy + size.height + 2,
              child: Material(
                elevation: 6,
                color: Colors.white,
                borderRadius: BorderRadius.circular(4),
                child: TableSizePicker(
                  onSelected: (value) => Navigator.pop(context, value),
                ),
              ),
            ),
          ],
        );
      },
    );
  }

  @override
  State<TableSizePicker> createState() => _TableSizePickerState();
}

class _TableSizePickerState extends State<TableSizePicker> {
  int _rows = 0;
  int _cols = 0;

  @override
  Widget build(BuildContext context) {
    final label = _rows == 0 || _cols == 0
        ? 'Insert Table'
        : '$_rows × $_cols Table';

    return Padding(
      padding: const EdgeInsets.fromLTRB(10, 8, 10, 10),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: WordTheme.ribbonLabel),
          const SizedBox(height: 8),
          Column(
            children: List.generate(widget.maxRows, (r) {
              return Row(
                mainAxisSize: MainAxisSize.min,
                children: List.generate(widget.maxCols, (c) {
                  final selected = r < _rows && c < _cols;
                  return MouseRegion(
                    onEnter: (_) => setState(() {
                      _rows = r + 1;
                      _cols = c + 1;
                    }),
                    child: GestureDetector(
                      onTap: () => widget.onSelected((rows: r + 1, cols: c + 1)),
                      child: Container(
                        key: Key('table_size_cell_${r + 1}_${c + 1}'),
                        width: 18,
                        height: 18,
                        margin: const EdgeInsets.all(1),
                        decoration: BoxDecoration(
                          color: selected
                              ? const Color(0xFFCDE6F7)
                              : Colors.white,
                          border: Border.all(
                            color: selected
                                ? const Color(0xFF2B579A)
                                : const Color(0xFFB0B0B0),
                          ),
                        ),
                      ),
                    ),
                  );
                }),
              );
            }),
          ),
        ],
      ),
    );
  }
}
