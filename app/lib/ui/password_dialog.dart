import 'package:flutter/material.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Prompt for a password when opening an encrypted document (F22.S1).
class PasswordDialog extends StatefulWidget {
  const PasswordDialog({
    super.key,
    this.fileName,
    this.errorMessage,
  });

  final String? fileName;
  final String? errorMessage;

  static Future<String?> show(
    BuildContext context, {
    String? fileName,
    String? errorMessage,
  }) {
    return showDialog<String>(
      context: context,
      barrierDismissible: true,
      builder: (context) => PasswordDialog(
        fileName: fileName,
        errorMessage: errorMessage,
      ),
    );
  }

  @override
  State<PasswordDialog> createState() => _PasswordDialogState();
}

class _PasswordDialogState extends State<PasswordDialog> {
  late final TextEditingController _controller;
  var _obscure = true;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _submit() {
    final password = _controller.text;
    if (password.isEmpty) return;
    Navigator.pop(context, password);
  }

  @override
  Widget build(BuildContext context) {
    final name = widget.fileName;
    return AlertDialog(
      key: const Key('password_dialog'),
      title: const Text('Password'),
      content: SizedBox(
        width: 360,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              name == null || name.isEmpty
                  ? 'This document is password protected. Enter the password to open it.'
                  : '“$name” is password protected. Enter the password to open it.',
              style: const TextStyle(fontSize: 13, color: WordTheme.ribbonText),
            ),
            if (widget.errorMessage != null && widget.errorMessage!.isNotEmpty) ...[
              const SizedBox(height: 10),
              Text(
                widget.errorMessage!,
                style: const TextStyle(fontSize: 12, color: Color(0xFFC42B1C)),
              ),
            ],
            const SizedBox(height: 14),
            TextField(
              key: const Key('password_dialog_field'),
              controller: _controller,
              obscureText: _obscure,
              autofocus: true,
              onSubmitted: (_) => _submit(),
              decoration: InputDecoration(
                labelText: 'Password',
                border: const OutlineInputBorder(),
                suffixIcon: IconButton(
                  tooltip: _obscure ? 'Show password' : 'Hide password',
                  onPressed: () => setState(() => _obscure = !_obscure),
                  icon: Icon(_obscure ? Icons.visibility : Icons.visibility_off),
                ),
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        FilledButton(
          key: const Key('password_dialog_ok'),
          onPressed: _submit,
          child: const Text('OK'),
        ),
      ],
    );
  }
}
