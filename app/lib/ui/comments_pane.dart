import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/comment_thread.dart';
import 'package:tutuaword/editor/editor_controller.dart';

/// Lightweight comments pane — list threads, reply, resolve (L3/L5).
class CommentsPane extends StatefulWidget {
  const CommentsPane({super.key, required this.controller});

  final EditorController controller;

  static Future<void> show(BuildContext context, EditorController controller) {
    return showDialog<void>(
      context: context,
      builder: (context) => CommentsPane(controller: controller),
    );
  }

  @override
  State<CommentsPane> createState() => _CommentsPaneState();
}

class _CommentsPaneState extends State<CommentsPane> {
  late Future<CommentThreadList> _threads;

  @override
  void initState() {
    super.initState();
    _reload();
  }

  void _reload() {
    _threads = widget.controller.loadCommentThreads();
  }

  Future<void> _reply(CommentThreadView thread) async {
    final body = await showDialog<String>(
      context: context,
      builder: (context) {
        final controller = TextEditingController();
        return AlertDialog(
          title: Text('Reply to comment ${thread.commentId}'),
          content: TextField(
            controller: controller,
            autofocus: true,
            maxLines: 3,
            decoration: const InputDecoration(
              labelText: 'Reply',
              border: OutlineInputBorder(),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Cancel'),
            ),
            FilledButton(
              onPressed: () => Navigator.of(context).pop(controller.text.trim()),
              child: const Text('Reply'),
            ),
          ],
        );
      },
    );
    if (body == null || body.isEmpty) return;
    await widget.controller.replyToComment(thread.commentId, body);
    setState(_reload);
  }

  Future<void> _resolve(CommentThreadView thread) async {
    await widget.controller.resolveComment(thread.commentId, resolved: true);
    setState(_reload);
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('comments_pane'),
      title: const Text('Comments'),
      content: SizedBox(
        width: 420,
        height: 360,
        child: FutureBuilder<CommentThreadList>(
          future: _threads,
          builder: (context, snapshot) {
            final threads = snapshot.data?.threads ?? const [];
            if (snapshot.connectionState != ConnectionState.done) {
              return const Center(child: CircularProgressIndicator());
            }
            if (threads.isEmpty) {
              return const Center(child: Text('No comments yet'));
            }
            return ListView.separated(
              itemCount: threads.length,
              separatorBuilder: (_, __) => const Divider(height: 12),
              itemBuilder: (context, index) {
                final thread = threads[index];
                return ListTile(
                  key: Key('comment_thread_${thread.commentId}'),
                  title: Text(thread.preview),
                  subtitle: Text(
                    thread.resolved
                        ? 'Resolved · ${thread.messages.length} message(s)'
                        : '${thread.messages.length} message(s)',
                  ),
                  trailing: Wrap(
                    spacing: 4,
                    children: [
                      if (!thread.resolved)
                        IconButton(
                          key: Key('resolve_comment_${thread.commentId}'),
                          tooltip: 'Resolve',
                          icon: const Icon(Icons.check_circle_outline),
                          onPressed: () => _resolve(thread),
                        ),
                      IconButton(
                        key: Key('reply_comment_${thread.commentId}'),
                        tooltip: 'Reply',
                        icon: const Icon(Icons.reply_outlined),
                        onPressed: () => _reply(thread),
                      ),
                    ],
                  ),
                );
              },
            );
          },
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Close'),
        ),
      ],
    );
  }
}
