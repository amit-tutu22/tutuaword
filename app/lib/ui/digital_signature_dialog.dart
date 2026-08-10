import 'package:flutter/material.dart';
import 'package:tutuaword/bridge/digital_signature.dart';
import 'package:tutuaword/ui/word_theme.dart';

/// Result from the Sign Document dialog (F22.S4).
class DigitalSignatureSignRequest {
  const DigitalSignatureSignRequest({
    required this.name,
    this.email = '',
    this.organization,
  });

  final String name;
  final String email;
  final String? organization;
}

/// Dialog to sign a document or review existing signatures (F22.S4).
class DigitalSignatureDialog extends StatefulWidget {
  const DigitalSignatureDialog({
    super.key,
    required this.signatures,
    required this.verifications,
  });

  final List<DigitalSignatureInfo> signatures;
  final List<SignatureVerificationInfo> verifications;

  /// Returns a [DigitalSignatureSignRequest], `'clear'` to remove all, or null.
  static Future<Object?> show(
    BuildContext context, {
    required List<DigitalSignatureInfo> signatures,
    required List<SignatureVerificationInfo> verifications,
  }) {
    return showDialog<Object?>(
      context: context,
      barrierDismissible: true,
      builder: (context) => DigitalSignatureDialog(
        signatures: signatures,
        verifications: verifications,
      ),
    );
  }

  @override
  State<DigitalSignatureDialog> createState() => _DigitalSignatureDialogState();
}

class _DigitalSignatureDialogState extends State<DigitalSignatureDialog> {
  late final TextEditingController _nameController;
  late final TextEditingController _emailController;
  late final TextEditingController _orgController;
  String? _error;

  @override
  void initState() {
    super.initState();
    _nameController = TextEditingController();
    _emailController = TextEditingController();
    _orgController = TextEditingController();
  }

  @override
  void dispose() {
    _nameController.dispose();
    _emailController.dispose();
    _orgController.dispose();
    super.dispose();
  }

  void _sign() {
    final name = _nameController.text.trim();
    if (name.isEmpty) {
      setState(() => _error = 'Enter a signer name.');
      return;
    }
    final org = _orgController.text.trim();
    Navigator.pop(
      context,
      DigitalSignatureSignRequest(
        name: name,
        email: _emailController.text.trim(),
        organization: org.isEmpty ? null : org,
      ),
    );
  }

  SignatureVerificationInfo? _verificationFor(String id) {
    for (final v in widget.verifications) {
      if (v.signatureId == id) return v;
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      key: const Key('digital_signature_dialog'),
      title: const Text('Digital Signatures'),
      content: SizedBox(
        width: 420,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              if (widget.signatures.isEmpty)
                const Text(
                  'This document has no digital signatures.',
                  style: TextStyle(fontSize: 13, color: WordTheme.ribbonText),
                )
              else ...[
                Text(
                  '${widget.signatures.length} signature'
                  '${widget.signatures.length == 1 ? '' : 's'}',
                  style: const TextStyle(
                    fontSize: 12,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 8),
                for (final sig in widget.signatures)
                  _SignatureTile(
                    signature: sig,
                    verification: _verificationFor(sig.id),
                  ),
                const SizedBox(height: 12),
              ],
              const Text(
                'Sign this document',
                style: TextStyle(fontSize: 12, fontWeight: FontWeight.w600),
              ),
              const SizedBox(height: 6),
              const Text(
                'Creates an Ed25519 signature over the current document content.',
                style: TextStyle(fontSize: 12, color: WordTheme.ribbonText),
              ),
              if (_error != null) ...[
                const SizedBox(height: 8),
                Text(
                  _error!,
                  key: const Key('digital_signature_error'),
                  style: const TextStyle(fontSize: 12, color: Color(0xFFC42B1C)),
                ),
              ],
              const SizedBox(height: 10),
              TextField(
                key: const Key('digital_signature_name'),
                controller: _nameController,
                autofocus: true,
                decoration: const InputDecoration(
                  labelText: 'Name',
                  border: OutlineInputBorder(),
                ),
              ),
              const SizedBox(height: 10),
              TextField(
                key: const Key('digital_signature_email'),
                controller: _emailController,
                decoration: const InputDecoration(
                  labelText: 'Email (optional)',
                  border: OutlineInputBorder(),
                ),
              ),
              const SizedBox(height: 10),
              TextField(
                key: const Key('digital_signature_org'),
                controller: _orgController,
                decoration: const InputDecoration(
                  labelText: 'Organization (optional)',
                  border: OutlineInputBorder(),
                ),
              ),
            ],
          ),
        ),
      ),
      actions: [
        if (widget.signatures.isNotEmpty)
          TextButton(
            key: const Key('digital_signature_clear'),
            onPressed: () => Navigator.pop(context, 'clear'),
            child: const Text('Remove All'),
          ),
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
        FilledButton(
          key: const Key('digital_signature_sign'),
          onPressed: _sign,
          child: const Text('Sign'),
        ),
      ],
    );
  }
}

class _SignatureTile extends StatelessWidget {
  const _SignatureTile({
    required this.signature,
    this.verification,
  });

  final DigitalSignatureInfo signature;
  final SignatureVerificationInfo? verification;

  @override
  Widget build(BuildContext context) {
    final status = verification?.status ?? 'unknown';
    final Color statusColor;
    switch (status) {
      case 'valid':
        statusColor = const Color(0xFF107C10);
        break;
      case 'tampered':
        statusColor = const Color(0xFFC42B1C);
        break;
      default:
        statusColor = const Color(0xFFCA5010);
    }
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Container(
        width: double.infinity,
        padding: const EdgeInsets.all(10),
        decoration: BoxDecoration(
          border: Border.all(color: const Color(0xFFD0D0D0)),
          borderRadius: BorderRadius.circular(4),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              signature.signerName,
              style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w600),
            ),
            if (signature.signerEmail.isNotEmpty)
              Text(
                signature.signerEmail,
                style: const TextStyle(fontSize: 11, color: Color(0xFF555555)),
              ),
            const SizedBox(height: 4),
            Text(
              verification?.message ?? status,
              style: TextStyle(fontSize: 11, color: statusColor),
            ),
          ],
        ),
      ),
    );
  }
}
