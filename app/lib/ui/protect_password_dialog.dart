import 'package:flutter/material.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Dialog to set a password that encrypts DOCX on save (F22.S2).
class ProtectPasswordDialog extends StatefulWidget {
  const ProtectPasswordDialog({super.key});

  static Future<String?> show(BuildContext context) {
    return showDialog<String>(
      context: context,
      barrierDismissible: true,
      builder: (context) => const ProtectPasswordDialog(),
    );
  }

  @override
  State<ProtectPasswordDialog> createState() => _ProtectPasswordDialogState();
}

class _ProtectPasswordDialogState extends State<ProtectPasswordDialog> {
  late final TextEditingController _passwordController;
  late final TextEditingController _confirmController;
  var _obscure = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _passwordController = TextEditingController();
    _confirmController = TextEditingController();
  }

  @override
  void dispose() {
    _passwordController.dispose();
    _confirmController.dispose();
    super.dispose();
  }

  void _submit() {
    final password = _passwordController.text;
    final confirm = _confirmController.text;
    if (password.isEmpty) {
      setState(() => _error = 'Enter a password.');
      return;
    }
    if (password != confirm) {
      setState(() => _error = 'Passwords do not match.');
      return;
    }
    Navigator.pop(context, password);
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('protect_password_dialog'),
      title: const Text('Protect with Password'),
      content: SizedBox(
        width: 360,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Encrypt this document when saving as DOCX. You will need the password to open it later.',
              style: TextStyle(fontSize: 13, color: WordTheme.ribbonText),
            ),
            if (_error != null) ...[
              const SizedBox(height: 10),
              Text(
                _error!,
                key: const Key('protect_password_error'),
                style: const TextStyle(fontSize: 12, color: Color(0xFFC42B1C)),
              ),
            ],
            const SizedBox(height: 14),
            TextField(
              key: const Key('protect_password_field'),
              controller: _passwordController,
              obscureText: _obscure,
              autofocus: true,
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
            const SizedBox(height: 12),
            TextField(
              key: const Key('protect_password_confirm_field'),
              controller: _confirmController,
              obscureText: _obscure,
              onSubmitted: (_) => _submit(),
              decoration: const InputDecoration(
                labelText: 'Confirm password',
                border: OutlineInputBorder(),
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
          key: const Key('protect_password_ok'),
          onPressed: _submit,
          child: const Text('OK'),
        ),
      ],
    );
  }
}
