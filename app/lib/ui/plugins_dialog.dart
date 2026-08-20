import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/plugin_registry.dart';

/// Manage installed WASM / host plugins (F26.S3).
class PluginsDialog extends StatefulWidget {
  const PluginsDialog({
    super.key,
    required this.registry,
    this.onChanged,
    this.onInstallSample,
    this.onInvoke,
  });

  final PluginRegistry registry;
  final VoidCallback? onChanged;
  final void Function(bool grantEdit)? onInstallSample;
  final String Function(String id)? onInvoke;

  static Future<void> show(
    BuildContext context, {
    required PluginRegistry registry,
    VoidCallback? onChanged,
    void Function(bool grantEdit)? onInstallSample,
    String Function(String id)? onInvoke,
  }) {
    return showDialog<void>(
      context: context,
      barrierDismissible: true,
      builder: (context) => PluginsDialog(
        registry: registry,
        onChanged: onChanged,
        onInstallSample: onInstallSample,
        onInvoke: onInvoke,
      ),
    );
  }

  @override
  State<PluginsDialog> createState() => _PluginsDialogState();
}

class _PluginsDialogState extends State<PluginsDialog> {
  String? _status;

  void _refresh([String? status]) {
    setState(() => _status = status);
    widget.onChanged?.call();
  }

  @override
  Widget build(BuildContext context) {
    final plugins = widget.registry.list();
    return AlertDialog(
      key: const Key('plugins_dialog'),
      title: const Text('Plugins'),
      content: SizedBox(
        width: 420,
        height: 320,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Text(
                'WASM plugins run in a wasmtime sandbox with capability gates.',
              ),
              const SizedBox(height: 12),
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  FilledButton(
                    key: const Key('plugins_install_sample'),
                    onPressed: () {
                      if (widget.onInstallSample != null) {
                        widget.onInstallSample!(true);
                      } else {
                        widget.registry.installSampleEditPlugin(grantEdit: true);
                      }
                      _refresh(
                        'Installed Sample Edit Plugin '
                        '(granted: document.read, document.edit)',
                      );
                    },
                    child: const Text('Install sample'),
                  ),
                  OutlinedButton(
                    key: const Key('plugins_install_sample_readonly'),
                    onPressed: () {
                      if (widget.onInstallSample != null) {
                        widget.onInstallSample!(false);
                      } else {
                        widget.registry.installSampleEditPlugin(grantEdit: false);
                      }
                      _refresh(
                        'Installed sample read-only '
                        '(document.edit not granted — Run will be denied)',
                      );
                    },
                    child: const Text('Install read-only'),
                  ),
                ],
              ),
              const SizedBox(height: 12),
              if (plugins.isEmpty)
                const Text('No plugins installed.')
              else
                ...plugins.map((p) {
                  return ListTile(
                    key: Key('plugin_row_${p.id}'),
                    contentPadding: EdgeInsets.zero,
                    title: Text(p.name),
                    subtitle: Text(
                      '${p.id} · ${p.enabled ? "enabled" : "disabled"}\n'
                      'Granted: ${p.grantedLabel}',
                    ),
                    isThreeLine: true,
                    trailing: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        TextButton(
                          key: Key('plugin_invoke_${p.id}'),
                          onPressed: () {
                            final result = widget.onInvoke != null
                                ? widget.onInvoke!(p.id)
                                : widget.registry.invoke(p.id);
                            _refresh(result);
                          },
                          child: const Text('Run'),
                        ),
                        IconButton(
                          key: Key('plugin_toggle_${p.id}'),
                          tooltip: p.enabled ? 'Disable' : 'Enable',
                          onPressed: () {
                            if (p.enabled) {
                              widget.registry.disable(p.id);
                            } else {
                              widget.registry.enable(p.id);
                            }
                            _refresh(
                              p.enabled
                                  ? 'Disabled ${p.id}'
                                  : 'Enabled ${p.id}',
                            );
                          },
                          icon: Icon(
                            p.enabled
                                ? Icons.pause_circle_outline
                                : Icons.play_circle_outline,
                          ),
                        ),
                      ],
                    ),
                  );
                }),
              if (_status != null) ...[
                const SizedBox(height: 8),
                Text(
                  key: const Key('plugins_status'),
                  _status!,
                  style: TextStyle(
                    color: _status!.startsWith('ERR:')
                        ? Theme.of(context).colorScheme.error
                        : null,
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          key: const Key('plugins_close_button'),
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
      ],
    );
  }
}
