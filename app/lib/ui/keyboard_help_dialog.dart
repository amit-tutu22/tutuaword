import 'package:flutter/material.dart';
import 'package:tutuaword/ui/keyboard_shortcuts.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// One chord/action pair shown in the Help dialog.
class KeyboardHelpRow {
  const KeyboardHelpRow({required this.chord, required this.action});

  final String chord;
  final String action;
}

/// A titled group of shortcuts, matching the sections in `docs/keyboard.md`.
class KeyboardHelpSection {
  const KeyboardHelpSection({required this.title, required this.rows});

  final String title;
  final List<KeyboardHelpRow> rows;

  KeyboardHelpSection filtered(String query) {
    final q = query.trim().toLowerCase();
    if (q.isEmpty) return this;
    return KeyboardHelpSection(
      title: title,
      rows: rows
          .where(
            (row) =>
                row.chord.toLowerCase().contains(q) ||
                row.action.toLowerCase().contains(q),
          )
          .toList(),
    );
  }
}

/// Builds the Help catalog from the wired shortcut tables so the dialog cannot
/// drift from what the app actually handles.
List<KeyboardHelpSection> buildKeyboardHelpSections() {
  List<KeyboardHelpRow> fromChords(Set<WordChord> chords) {
    final rows = <KeyboardHelpRow>[];
    final seen = <String>{};
    for (final spec in kWordChordSpecs) {
      if (!chords.contains(spec.chord)) continue;
      final key = '${spec.label}|${spec.action}';
      if (!seen.add(key)) continue;
      rows.add(KeyboardHelpRow(chord: spec.label, action: spec.action));
    }
    return rows;
  }

  return [
    KeyboardHelpSection(
      title: 'File',
      rows: fromChords({
        WordChord.newDocument,
        WordChord.openDocument,
        WordChord.save,
        WordChord.saveAs,
        WordChord.print,
        WordChord.printPreview,
        WordChord.keyboardHelp,
      }),
    ),
    KeyboardHelpSection(
      title: 'Undo and clipboard',
      rows: fromChords({
        WordChord.undo,
        WordChord.redo,
        WordChord.cut,
        WordChord.copy,
        WordChord.paste,
        WordChord.copyFormatting,
        WordChord.pasteFormatting,
        WordChord.pasteSpecial,
        WordChord.pasteMatchStyle,
      }),
    ),
    KeyboardHelpSection(
      title: 'Select, find and navigate',
      rows: fromChords({
        WordChord.selectAll,
        WordChord.find,
        WordChord.replace,
        WordChord.findNext,
        WordChord.goTo,
        WordChord.formattingMarks,
      }),
    ),
    KeyboardHelpSection(
      title: 'Character formatting',
      rows: fromChords({
        WordChord.bold,
        WordChord.italic,
        WordChord.underline,
        WordChord.growFont,
        WordChord.shrinkFont,
        WordChord.subscript,
        WordChord.superscript,
        WordChord.allCaps,
        WordChord.smallCaps,
        WordChord.strikethrough,
        WordChord.hiddenText,
        WordChord.changeCase,
        WordChord.clearFormatting,
      }),
    ),
    KeyboardHelpSection(
      title: 'Paragraph formatting',
      rows: fromChords({
        WordChord.alignLeft,
        WordChord.alignCenter,
        WordChord.alignRight,
        WordChord.alignJustify,
        WordChord.increaseIndent,
        WordChord.decreaseIndent,
        WordChord.promoteOutline,
        WordChord.demoteOutline,
        WordChord.moveParagraphUp,
        WordChord.moveParagraphDown,
        WordChord.clearParagraphFormatting,
        WordChord.hangingIndent,
        WordChord.removeHangingIndent,
        WordChord.toggleSpaceBefore,
        WordChord.bulletList,
        WordChord.singleSpace,
        WordChord.doubleSpace,
        WordChord.oneAndAHalfSpace,
      }),
    ),
    KeyboardHelpSection(
      title: 'Styles',
      rows: fromChords({
        WordChord.normalStyle,
        WordChord.heading1,
        WordChord.heading2,
        WordChord.heading3,
      }),
    ),
    KeyboardHelpSection(
      title: 'Insert and review',
      rows: fromChords({
        WordChord.hyperlink,
        WordChord.comment,
        WordChord.trackChanges,
        WordChord.footnote,
        WordChord.endnote,
        WordChord.dateField,
        WordChord.pageNumberField,
        WordChord.spelling,
      }),
    ),
    KeyboardHelpSection(
      title: 'Characters that are hard to type',
      rows: fromChords({
        WordChord.nonbreakingSpace,
        WordChord.nonbreakingHyphen,
        WordChord.optionalHyphen,
        WordChord.copyright,
        WordChord.registered,
        WordChord.trademark,
        WordChord.ellipsis,
      }),
    ),
    KeyboardHelpSection(
      title: 'Editor (document canvas)',
      rows: [
        for (final entry in kEditorKeyActions.entries)
          KeyboardHelpRow(chord: entry.key, action: entry.value),
      ],
    ),
  ];
}

/// In-app keyboard map — title-bar gear, Help menu, About, and F1.
class KeyboardHelpDialog extends StatefulWidget {
  const KeyboardHelpDialog({super.key});

  static Future<void> show(BuildContext context) {
    return showDialog<void>(
      context: context,
      barrierDismissible: true,
      builder: (context) => const KeyboardHelpDialog(),
    );
  }

  @override
  State<KeyboardHelpDialog> createState() => _KeyboardHelpDialogState();
}

class _KeyboardHelpDialogState extends State<KeyboardHelpDialog> {
  final _search = TextEditingController();
  late final List<KeyboardHelpSection> _all = buildKeyboardHelpSections();

  @override
  void dispose() {
    _search.dispose();
    super.dispose();
  }

  List<KeyboardHelpSection> get _visible {
    final query = _search.text;
    return [
      for (final section in _all)
        section.filtered(query),
    ].where((section) => section.rows.isNotEmpty).toList();
  }

  @override
  Widget build(BuildContext context) {
    final size = MediaQuery.sizeOf(context);
    final width = size.width < 720 ? size.width - 32.0 : 640.0;
    final height = size.height < 720 ? size.height * 0.85 : 560.0;

    return AlertDialog(
      key: const Key('keyboard_help_dialog'),
      title: const Text('Keyboard shortcuts'),
      contentPadding: const EdgeInsets.fromLTRB(20, 12, 20, 0),
      content: SizedBox(
        width: width,
        height: height,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              'Word-compatible chords wired in Tutuaword. Ctrl and Cmd both work '
              'for every application shortcut.',
              style: WordTheme.ribbonLabel.copyWith(
                fontSize: 12,
                color: const Color(0xFF555555),
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              key: const Key('keyboard_help_search'),
              controller: _search,
              autofocus: true,
              decoration: const InputDecoration(
                isDense: true,
                prefixIcon: Icon(Icons.search, size: 20),
                hintText: 'Search shortcuts…',
                border: OutlineInputBorder(),
              ),
              onChanged: (_) => setState(() {}),
            ),
            const SizedBox(height: 8),
            Expanded(
              child: _visible.isEmpty
                  ? const Center(
                      child: Text(
                        'No shortcuts match',
                        style: TextStyle(color: Color(0xFF777777)),
                      ),
                    )
                  : ListView.builder(
                      key: const Key('keyboard_help_list'),
                      itemCount: _visible.length,
                      itemBuilder: (context, index) {
                        final section = _visible[index];
                        return _HelpSectionCard(section: section);
                      },
                    ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('keyboard_help_close'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Close'),
        ),
      ],
    );
  }
}

class _HelpSectionCard extends StatelessWidget {
  const _HelpSectionCard({required this.section});

  final KeyboardHelpSection section;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            section.title,
            style: Theme.of(context).textTheme.titleSmall?.copyWith(
                  fontWeight: FontWeight.w600,
                  color: WordTheme.ribbonText,
                ),
          ),
          const SizedBox(height: 6),
          DecoratedBox(
            decoration: BoxDecoration(
              border: Border.all(color: WordTheme.groupDivider),
              borderRadius: BorderRadius.circular(6),
              color: Colors.white,
            ),
            child: Column(
              children: [
                for (var i = 0; i < section.rows.length; i++) ...[
                  if (i > 0)
                    const Divider(height: 1, color: WordTheme.groupDivider),
                  _HelpRowTile(row: section.rows[i]),
                ],
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _HelpRowTile extends StatelessWidget {
  const _HelpRowTile({required this.row});

  final KeyboardHelpRow row;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            flex: 5,
            child: Text(
              row.action,
              style: const TextStyle(
                fontSize: 13,
                color: WordTheme.ribbonText,
              ),
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            flex: 4,
            child: Align(
              alignment: Alignment.centerRight,
              child: Text(
                row.chord,
                textAlign: TextAlign.right,
                style: const TextStyle(
                  fontSize: 12,
                  fontFamily: 'Menlo',
                  fontFamilyFallback: ['Consolas', 'monospace'],
                  color: Color(0xFF444444),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
