import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';
import 'package:tutuaword/ui/word_theme.dart';

class FindPane extends StatefulWidget {
  const FindPane({super.key, required this.controller});

  final EditorController controller;

  @override
  State<FindPane> createState() => _FindPaneState();
}

class _FindPaneState extends State<FindPane> {
  late final TextEditingController _findController;
  late final TextEditingController _replaceController;
  late final FocusNode _focusNode;

  @override
  void initState() {
    super.initState();
    _findController = TextEditingController(text: widget.controller.findQuery);
    _replaceController = TextEditingController(text: widget.controller.findReplaceText);
    _focusNode = FocusNode();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) _focusNode.requestFocus();
    });
  }

  @override
  void dispose() {
    _findController.dispose();
    _replaceController.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final status = widget.controller.findStatusText;
    final phone = WordTheme.phoneChrome(context);
    return Material(
      key: const Key('find_pane'),
      elevation: 2,
      color: WordTheme.ribbonSurface,
      child: phone
          ? _buildPhoneLayout(context, status)
          : _buildDesktopLayout(context, status),
    );
  }

  Widget _buildPhoneLayout(BuildContext context, String status) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 8, 12, 8),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              const Expanded(
                child: Text('Find & Replace', style: WordTheme.ribbonLabel),
              ),
              IconButton(
                key: const Key('find_close'),
                tooltip: 'Close',
                icon: const Icon(Icons.close, size: 18),
                onPressed: widget.controller.closeFindPane,
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
              ),
            ],
          ),
          const SizedBox(height: 6),
          SizedBox(
            height: 32,
            child: TextField(
              key: const Key('find_query'),
              controller: _findController,
              focusNode: _focusNode,
              decoration: const InputDecoration(
                isDense: true,
                hintText: 'Find',
                contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 8),
                border: OutlineInputBorder(),
              ),
              onChanged: widget.controller.setFindQuery,
              onSubmitted: (_) => widget.controller.findNext(),
            ),
          ),
          const SizedBox(height: 6),
          SizedBox(
            height: 32,
            child: TextField(
              key: const Key('find_replace'),
              controller: _replaceController,
              decoration: const InputDecoration(
                isDense: true,
                hintText: 'Replace with',
                contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 8),
                border: OutlineInputBorder(),
              ),
              onChanged: widget.controller.setFindReplaceText,
            ),
          ),
          const SizedBox(height: 6),
          SingleChildScrollView(
            scrollDirection: Axis.horizontal,
            child: Row(
              children: [
                FilterChip(
                  key: const Key('find_match_case'),
                  label: const Text('Match case', style: TextStyle(fontSize: 11)),
                  selected: widget.controller.findMatchCase,
                  onSelected: (_) => widget.controller.toggleFindMatchCase(),
                  visualDensity: VisualDensity.compact,
                ),
                FilterChip(
                  key: const Key('find_use_regex'),
                  label: const Text('Regex', style: TextStyle(fontSize: 11)),
                  selected: widget.controller.findUseRegex,
                  onSelected: (_) => widget.controller.toggleFindUseRegex(),
                  visualDensity: VisualDensity.compact,
                ),
                FilterChip(
                  key: const Key('find_use_wildcards'),
                  label: const Text('Wildcards', style: TextStyle(fontSize: 11)),
                  selected: widget.controller.findUseWildcards,
                  onSelected: (_) => widget.controller.toggleFindUseWildcards(),
                  visualDensity: VisualDensity.compact,
                ),
              ],
            ),
          ),
          const SizedBox(height: 6),
          Row(
            children: [
              IconButton(
                key: const Key('find_previous'),
                tooltip: 'Previous',
                icon: const Icon(Icons.arrow_upward, size: 18),
                onPressed: widget.controller.findPrevious,
              ),
              IconButton(
                key: const Key('find_next'),
                tooltip: 'Next',
                icon: const Icon(Icons.arrow_downward, size: 18),
                onPressed: widget.controller.findNext,
              ),
              TextButton(
                key: const Key('replace_all'),
                onPressed: () => widget.controller.replaceAll(),
                child: const Text('Replace All', style: TextStyle(fontSize: 11)),
              ),
              if (status.isNotEmpty)
                Expanded(
                  child: Text(
                    status,
                    style: WordTheme.ribbonLabel.copyWith(fontSize: 11),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildDesktopLayout(BuildContext context, String status) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      child: Row(
        children: [
          const Text('Find', style: WordTheme.ribbonLabel),
          const SizedBox(width: 8),
          SizedBox(
            width: 160,
            height: 28,
            child: TextField(
              key: const Key('find_query'),
              controller: _findController,
              focusNode: _focusNode,
              decoration: const InputDecoration(
                isDense: true,
                contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                border: OutlineInputBorder(),
              ),
              onChanged: widget.controller.setFindQuery,
              onSubmitted: (_) => widget.controller.findNext(),
            ),
          ),
          const SizedBox(width: 8),
          const Text('Replace', style: WordTheme.ribbonLabel),
          const SizedBox(width: 8),
          SizedBox(
            width: 160,
            height: 28,
            child: TextField(
              key: const Key('find_replace'),
              controller: _replaceController,
              decoration: const InputDecoration(
                isDense: true,
                contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                border: OutlineInputBorder(),
              ),
              onChanged: widget.controller.setFindReplaceText,
            ),
          ),
          const SizedBox(width: 8),
          FilterChip(
            key: const Key('find_match_case'),
            label: const Text('Match case', style: TextStyle(fontSize: 11)),
            selected: widget.controller.findMatchCase,
            onSelected: (_) => widget.controller.toggleFindMatchCase(),
            visualDensity: VisualDensity.compact,
          ),
          FilterChip(
            key: const Key('find_use_regex'),
            label: const Text('Regex', style: TextStyle(fontSize: 11)),
            selected: widget.controller.findUseRegex,
            onSelected: (_) => widget.controller.toggleFindUseRegex(),
            visualDensity: VisualDensity.compact,
          ),
          FilterChip(
            key: const Key('find_use_wildcards'),
            label: const Text('Wildcards', style: TextStyle(fontSize: 11)),
            selected: widget.controller.findUseWildcards,
            onSelected: (_) => widget.controller.toggleFindUseWildcards(),
            visualDensity: VisualDensity.compact,
          ),
          FilterChip(
            key: const Key('find_format_bold'),
            label: const Text('Bold', style: TextStyle(fontSize: 11)),
            selected: widget.controller.findBold,
            onSelected: (_) => widget.controller.toggleFindBold(),
            visualDensity: VisualDensity.compact,
          ),
          SizedBox(
            width: 100,
            height: 28,
            child: TextField(
              key: const Key('find_style_name'),
              decoration: const InputDecoration(
                isDense: true,
                hintText: 'Style',
                contentPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                border: OutlineInputBorder(),
              ),
              onChanged: widget.controller.setFindStyleName,
            ),
          ),
          const SizedBox(width: 8),
          IconButton(
            key: const Key('find_previous'),
            tooltip: 'Previous',
            icon: const Icon(Icons.arrow_upward, size: 18),
            onPressed: widget.controller.findPrevious,
          ),
          IconButton(
            key: const Key('find_next'),
            tooltip: 'Next',
            icon: const Icon(Icons.arrow_downward, size: 18),
            onPressed: widget.controller.findNext,
          ),
          TextButton(
            key: const Key('replace_all'),
            onPressed: () => widget.controller.replaceAll(),
            child: const Text('Replace All', style: TextStyle(fontSize: 11)),
          ),
          if (status.isNotEmpty) ...[
            const SizedBox(width: 8),
            Text(status, style: WordTheme.ribbonLabel.copyWith(fontSize: 11)),
          ],
          const SizedBox(width: 8),
          IconButton(
            key: const Key('find_close'),
            tooltip: 'Close',
            icon: const Icon(Icons.close, size: 18),
            onPressed: widget.controller.closeFindPane,
          ),
        ],
      ),
    );
  }
}
