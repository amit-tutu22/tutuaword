import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/ai_chat.dart';
import 'package:tutuaword/bridge/ai_client.dart';

/// Document chat UI with paragraph citations (F28.S3).
class AiChatDialog extends StatefulWidget {
  const AiChatDialog({
    super.key,
    required this.client,
    required this.documentText,
    this.onCitationTap,
  });

  final AiClient client;
  final String documentText;
  final ValueChanged<String>? onCitationTap;

  static Future<void> show(
    BuildContext context, {
    required AiClient client,
    required String documentText,
    ValueChanged<String>? onCitationTap,
  }) {
    return showDialog<void>(
      context: context,
      barrierDismissible: true,
      builder: (context) => AiChatDialog(
        client: client,
        documentText: documentText,
        onCitationTap: onCitationTap,
      ),
    );
  }

  @override
  State<AiChatDialog> createState() => _AiChatDialogState();
}

class _AiChatDialogState extends State<AiChatDialog> {
  late final AiDocumentChatSession _session;
  late final TextEditingController _input;
  bool _busy = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _session = AiDocumentChatSession.fromText(widget.documentText);
    _input = TextEditingController();
  }

  @override
  void dispose() {
    _input.dispose();
    super.dispose();
  }

  Future<void> _send() async {
    final q = _input.text.trim();
    if (q.isEmpty || _busy) return;
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      await _session.ask(widget.client, q);
      _input.clear();
    } catch (e) {
      _error = e.toString();
    }
    if (mounted) {
      setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('ai_chat_dialog'),
      title: const Text('Ask about this document'),
      content: SizedBox(
        width: 480,
        height: 420,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              key: const Key('ai_chat_chunk_count'),
              '${_session.index.length} paragraphs indexed',
            ),
            const SizedBox(height: 8),
            Expanded(
              child: ListView.builder(
                key: const Key('ai_chat_history'),
                itemCount: _session.history.length,
                itemBuilder: (context, index) {
                  final msg = _session.history[index];
                  final isUser = msg.role == AiChatRole.user;
                  return Padding(
                    padding: const EdgeInsets.only(bottom: 8),
                    child: Column(
                      crossAxisAlignment: isUser
                          ? CrossAxisAlignment.end
                          : CrossAxisAlignment.start,
                      children: [
                        Text(
                          key: Key('ai_chat_msg_$index'),
                          isUser ? 'You: ${msg.content}' : msg.content,
                        ),
                        if (msg.references.isNotEmpty)
                          Wrap(
                            spacing: 6,
                            children: [
                              for (final ref in msg.references)
                                ActionChip(
                                  key: Key('ai_chat_cite_${ref.paragraphId}'),
                                  label: Text(ref.paragraphId),
                                  tooltip: ref.excerpt,
                                  onPressed: widget.onCitationTap == null
                                      ? null
                                      : () => widget.onCitationTap!(
                                            ref.paragraphId,
                                          ),
                                ),
                            ],
                          ),
                      ],
                    ),
                  );
                },
              ),
            ),
            if (_error != null)
              Text(
                _error!,
                key: const Key('ai_chat_error'),
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    key: const Key('ai_chat_input'),
                    controller: _input,
                    enabled: !_busy,
                    decoration: const InputDecoration(
                      hintText: 'Ask a question…',
                      border: OutlineInputBorder(),
                    ),
                    onSubmitted: (_) => _send(),
                  ),
                ),
                const SizedBox(width: 8),
                FilledButton(
                  key: const Key('ai_chat_send'),
                  onPressed: _busy ? null : _send,
                  child: Text(_busy ? '…' : 'Send'),
                ),
              ],
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          key: const Key('ai_chat_close'),
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Close'),
        ),
      ],
    );
  }
}
