import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/ai_client.dart';

/// AI routing / provider settings (F28.S1). Capability calls stay provider-agnostic.
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

  @override
  void initState() {
    super.initState();
    _mode = widget.client.routingMode;
    _openaiKey = TextEditingController(text: widget.client.openaiApiKey);
    _geminiKey = TextEditingController(text: widget.client.geminiApiKey);
    _llamaEndpoint = TextEditingController(text: widget.client.llamaEndpoint);
  }

  @override
  void dispose() {
    _openaiKey.dispose();
    _geminiKey.dispose();
    _llamaEndpoint.dispose();
    super.dispose();
  }

  void _setMode(AiRoutingMode? mode) {
    if (mode == null) return;
    setState(() => _mode = mode);
    widget.client.setRoutingMode(mode);
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
                'Routing chooses local or cloud models automatically. '
                'Capability actions never hardcode a provider.',
              ),
              const SizedBox(height: 12),
              const Text(
                'Routing mode',
                style: TextStyle(fontWeight: FontWeight.w600),
              ),
              RadioListTile<AiRoutingMode>(
                key: const Key('ai_routing_automatic'),
                title: const Text('Automatic'),
                subtitle: const Text('Grammar → local; long summarize → cloud'),
                value: AiRoutingMode.automatic,
                groupValue: _mode,
                onChanged: _setMode,
              ),
              RadioListTile<AiRoutingMode>(
                key: const Key('ai_routing_always_local'),
                title: const Text('Always Local'),
                value: AiRoutingMode.alwaysLocal,
                groupValue: _mode,
                onChanged: _setMode,
              ),
              RadioListTile<AiRoutingMode>(
                key: const Key('ai_routing_always_cloud'),
                title: const Text('Always Cloud'),
                value: AiRoutingMode.alwaysCloud,
                groupValue: _mode,
                onChanged: _setMode,
              ),
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
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Close'),
        ),
      ],
    );
  }
}
