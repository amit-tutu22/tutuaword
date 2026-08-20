import 'package:flutter/material.dart';
import 'package:tutuaword/editor/editor_controller.dart';

/// Tracked-change entry from engine JSON (F17.S2).
class TrackedChangeEntry {
  const TrackedChangeEntry({
    required this.runId,
    required this.revisionType,
    required this.author,
    required this.preview,
  });

  final String runId;
  final String revisionType;
  final String author;
  final String preview;

  factory TrackedChangeEntry.fromJson(Map<String, dynamic> json) {
    return TrackedChangeEntry(
      runId: json['run_id'] as String? ?? '',
      revisionType: json['revision_type'] as String? ?? 'insert',
      author: json['author'] as String? ?? '',
      preview: json['preview'] as String? ?? '',
    );
  }

  String get typeLabel => revisionType == 'delete' ? 'Deleted' : 'Inserted';
}

/// Side pane listing tracked changes with jump / accept / reject (F17.S2).
class ChangesPane extends StatelessWidget {
  const ChangesPane({
    super.key,
    required this.controller,
    this.expanded = false,
  });

  final EditorController controller;
  final bool expanded;

  Future<void> _focusChange(BuildContext context, TrackedChangeEntry entry) async {
    controller.focusRevision(entry.runId);
  }

  Future<void> _accept(BuildContext context, TrackedChangeEntry entry) async {
    controller.focusRevision(entry.runId);
    controller.acceptRevisionAtCaret();
    controller.removeTrackedChangeLocally(entry.runId);
  }

  Future<void> _reject(BuildContext context, TrackedChangeEntry entry) async {
    controller.focusRevision(entry.runId);
    controller.rejectRevisionAtCaret();
    controller.removeTrackedChangeLocally(entry.runId);
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: controller,
      builder: (context, _) {
        final entries = controller.trackedChanges;
        return Container(
          key: const Key('changes_pane'),
          width: expanded ? double.infinity : 260,
          color: Theme.of(context).colorScheme.surfaceContainerHighest,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              if (!expanded) ...[
                Padding(
                  padding: const EdgeInsets.fromLTRB(12, 10, 8, 6),
                  child: Row(
                    children: [
                      const Expanded(
                        child: Text(
                          'Changes',
                          style: TextStyle(
                            fontSize: 12,
                            fontWeight: FontWeight.w600,
                          ),
                        ),
                      ),
                      Tooltip(
                        message: 'Close',
                        child: IconButton(
                          icon: const Icon(Icons.close, size: 18),
                          onPressed: controller.hideChangesPane,
                          padding: EdgeInsets.zero,
                          constraints: const BoxConstraints(
                            minWidth: 32,
                            minHeight: 32,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                const Divider(height: 1),
              ],
              Padding(
                padding: const EdgeInsets.fromLTRB(12, 8, 12, 4),
                child: Text(
                  entries.isEmpty
                      ? 'No tracked changes.'
                      : '${entries.length} change${entries.length == 1 ? '' : 's'}',
                  style: const TextStyle(fontSize: 11, color: Color(0xFF555555)),
                ),
              ),
              Expanded(
                child: ListView.builder(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  itemCount: entries.length,
                  itemBuilder: (context, index) {
                    final entry = entries[index];
                    final color = entry.revisionType == 'delete'
                        ? const Color(0xFFC42B1C)
                        : const Color(0xFF107C10);
                    return Material(
                      color: Colors.transparent,
                      child: InkWell(
                        key: Key('changes_entry_${entry.runId}_$index'),
                        onTap: () => _focusChange(context, entry),
                        child: Padding(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 4,
                            vertical: 8,
                          ),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Row(
                                children: [
                                  Expanded(
                                    child: Text(
                                      entry.typeLabel,
                                      style: TextStyle(
                                        fontSize: 11,
                                        fontWeight: FontWeight.w600,
                                        color: color,
                                      ),
                                    ),
                                  ),
                                  IconButton(
                                    key: Key('changes_accept_${entry.runId}'),
                                    icon: const Icon(Icons.check, size: 16),
                                    tooltip: 'Accept',
                                    onPressed: () => _accept(context, entry),
                                    padding: EdgeInsets.zero,
                                    constraints: const BoxConstraints(
                                      minWidth: 28,
                                      minHeight: 28,
                                    ),
                                  ),
                                  IconButton(
                                    key: Key('changes_reject_${entry.runId}'),
                                    icon: const Icon(Icons.close, size: 16),
                                    tooltip: 'Reject',
                                    onPressed: () => _reject(context, entry),
                                    padding: EdgeInsets.zero,
                                    constraints: const BoxConstraints(
                                      minWidth: 28,
                                      minHeight: 28,
                                    ),
                                  ),
                                ],
                              ),
                              if (entry.author.isNotEmpty)
                                Text(
                                  entry.author,
                                  style: const TextStyle(
                                    fontSize: 10,
                                    color: Color(0xFF666666),
                                  ),
                                ),
                              const SizedBox(height: 2),
                              Text(
                                entry.preview.isEmpty ? '(empty)' : entry.preview,
                                maxLines: 3,
                                overflow: TextOverflow.ellipsis,
                                style: const TextStyle(fontSize: 11, height: 1.3),
                              ),
                            ],
                          ),
                        ),
                      ),
                    );
                  },
                ),
              ),
              Padding(
                padding: const EdgeInsets.all(8),
                child: TextButton.icon(
                  key: const Key('changes_pane_refresh'),
                  onPressed: controller.refreshRevisions,
                  icon: const Icon(Icons.refresh, size: 16),
                  label: const Text('Refresh'),
                ),
              ),
            ],
          ),
        );
      },
    );
  }
}
