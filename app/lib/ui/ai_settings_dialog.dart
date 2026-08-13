import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/ai_client.dart';
import 'package:tutuaword/bridge/ollama_host.dart';

/// AI routing / provider settings (F28.S1). Capability calls stay provider-agnostic.
///
/// Choosing **Always Local** (or Automatic) probes the local endpoint and, on
/// desktop, starts `ollama serve` when Ollama is installed but not running.
class AiSettingsDialog extends StatefulWidget {
  const AiSettingsDialog({
    super.key,
    required this.client,
    this.onChanged,
  });

  final AiClient client;
  final VoidCallback? onChanged;

  static Future<void> show(
    BuildContext context, {
    required AiClient client,
    VoidCallback? onChanged,
  }) {
    return showDialog<void>(
      context: context,
      barrierDismissible: true,
      builder: (context) => AiSettingsDialog(
        client: client,
        onChanged: onChanged,
      ),
    );
  }

  @override
  State<AiSettingsDialog> createState() => _AiSettingsDialogState();
}

class _AiSettingsDialogState extends State<AiSettingsDialog> {
  late AiRoutingMode _mode;
  late final TextEditingController _openaiKey;
  late final TextEditingController _geminiKey;
  late final TextEditingController _llamaEndpoint;
  bool _busy = false;
  String _status = '';

  @override
  void initState() {
    super.initState();
    _mode = widget.client.routingMode;
    _openaiKey = TextEditingController(text: widget.client.openaiApiKey);
    _geminiKey = TextEditingController(text: widget.client.geminiApiKey);
    _llamaEndpoint = TextEditingController(text: widget.client.llamaEndpoint);
    _status = widget.client.lastOllamaEnsure?.message ?? '';
  }

  @override
  void dispose() {
    _openaiKey.dispose();
    _geminiKey.dispose();
    _llamaEndpoint.dispose();
    super.dispose();
  }

  Future<void> _setMode(AiRoutingMode? mode) async {
    if (mode == null || _busy) return;
    setState(() {
      _mode = mode;
      _busy = mode != AiRoutingMode.alwaysCloud;
      _status = mode == AiRoutingMode.alwaysCloud
          ? 'Using cloud providers (OpenAI / Gemini).'
          : 'Starting local AI if needed…';
    });
    widget.client.openaiApiKey = _openaiKey.text.trim();
    widget.client.geminiApiKey = _geminiKey.text.trim();
    widget.client.llamaEndpoint = _llamaEndpoint.text.trim().isEmpty
        ? 'http://127.0.0.1:11434'
        : _llamaEndpoint.text.trim();

    final result = await widget.client.applyRoutingMode(mode);
    if (!mounted) return;
    setState(() {
      _busy = false;
      if (mode == AiRoutingMode.alwaysCloud) {
        _status = 'Using cloud providers (OpenAI / Gemini).';
      } else if (result == null) {
        _status = '';
      } else if (result.ok) {
        _status = result.status == OllamaEnsureStatus.started
            ? result.message
            : 'Local AI ready. ${result.message}';
      } else {
        _status = result.message;
      }
    });
    widget.onChanged?.call();
  }

  void _applyCredentials() {
    widget.client.openaiApiKey = _openaiKey.text.trim();
    widget.client.geminiApiKey = _geminiKey.text.trim();
    widget.client.llamaEndpoint = _llamaEndpoint.text.trim().isEmpty
        ? 'http://127.0.0.1:11434'
        : _llamaEndpoint.text.trim();
    setState(() {});
    widget.onChanged?.call();
  }

  @override
  Widget build(BuildContext context) {
    final providers = widget.client.listProviders();
    return AlertDialog(
      key: const Key('ai_settings_dialog'),
      title: const Text('AI Settings'),
      content: SizedBox(
        width: 420,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Text(
                'Choose where AI runs. Local uses Ollama on this machine '
                '(auto-started when you pick Local). Cloud uses your API keys.',
              ),
              const SizedBox(height: 12),
              const Text(
                'Mode',
                style: TextStyle(fontWeight: FontWeight.w600),
              ),
              RadioListTile<AiRoutingMode>(
                key: const Key('ai_routing_automatic'),
                title: const Text('Automatic'),
                subtitle: const Text(
                  'Short/local tasks → Ollama; heavy summarize → cloud',
                ),
                value: AiRoutingMode.automatic,
                groupValue: _mode,
                onChanged: _busy ? null : _setMode,
              ),
              RadioListTile<AiRoutingMode>(
                key: const Key('ai_routing_always_local'),
                title: const Text('Always Local'),
                subtitle: const Text(
                  'Starts Ollama automatically if installed (port 11434)',
                ),
                value: AiRoutingMode.alwaysLocal,
                groupValue: _mode,
                onChanged: _busy ? null : _setMode,
              ),
              RadioListTile<AiRoutingMode>(
                key: const Key('ai_routing_always_cloud'),
                title: const Text('Always Cloud'),
                subtitle: const Text('OpenAI or Gemini — no local server'),
                value: AiRoutingMode.alwaysCloud,
                groupValue: _mode,
                onChanged: _busy ? null : _setMode,
              ),
              if (_busy) ...[
                const SizedBox(height: 8),
                const LinearProgressIndicator(key: Key('ai_local_starting')),
              ],
              if (_status.isNotEmpty) ...[
                const SizedBox(height: 8),
                Text(
                  _status,
                  key: const Key('ai_local_status'),
                  style: TextStyle(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                    fontSize: 13,
                  ),
                ),
              ],
              const SizedBox(height: 8),
              const Text(
                'Credentials',
                style: TextStyle(fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 4),
              TextField(
                key: const Key('ai_openai_api_key'),
                controller: _openaiKey,
                obscureText: true,
                decoration: const InputDecoration(
                  labelText: 'OpenAI API key',
                  hintText: 'sk-… or OPENAI_API_KEY env',
                  isDense: true,
                ),
                onChanged: (_) => _applyCredentials(),
              ),
              const SizedBox(height: 8),
              TextField(
                key: const Key('ai_gemini_api_key'),
                controller: _geminiKey,
                obscureText: true,
                decoration: const InputDecoration(
                  labelText: 'Gemini API key',
                  hintText: 'GEMINI_API_KEY env',
                  isDense: true,
                ),
                onChanged: (_) => _applyCredentials(),
              ),
              const SizedBox(height: 8),
              TextField(
                key: const Key('ai_llama_endpoint'),
                controller: _llamaEndpoint,
                decoration: const InputDecoration(
                  labelText: 'Local llama / Ollama endpoint',
                  hintText: 'http://127.0.0.1:11434',
                  isDense: true,
                ),
                onChanged: (_) => _applyCredentials(),
              ),
              const SizedBox(height: 12),
              const Text(
                'Providers',
                style: TextStyle(fontWeight: FontWeight.w600),
              ),
              ...providers.map(
                (p) => ListTile(
                  key: Key('ai_provider_${p.id}'),
                  dense: true,
                  title: Text(p.name),
                  subtitle: Text(p.local ? 'Local' : 'Cloud'),
                  trailing: Text(
                    p.available ? 'Available' : 'Unavailable',
                    key: Key('ai_provider_status_${p.id}'),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          key: const Key('ai_settings_close'),
          onPressed: _busy ? null : () => Navigator.of(context).pop(),
          child: const Text('Close'),
        ),
      ],
    );
  }
}
