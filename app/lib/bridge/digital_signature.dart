/// One digital signature attached to a document (F22.S4).
class DigitalSignatureInfo {
  const DigitalSignatureInfo({
    required this.id,
    required this.signerName,
    required this.signerEmail,
    this.organization,
    required this.timestamp,
    required this.contentHash,
  });

  final String id;
  final String signerName;
  final String signerEmail;
  final String? organization;
  final String timestamp;
  final String contentHash;

  factory DigitalSignatureInfo.fromJson(Map<String, dynamic> json) {
    final signer = json['signer'];
    final signerMap =
        signer is Map ? Map<String, dynamic>.from(signer) : const <String, dynamic>{};
    return DigitalSignatureInfo(
      id: json['id'] as String? ?? '',
      signerName: signerMap['name'] as String? ?? '',
      signerEmail: signerMap['email'] as String? ?? '',
      organization: signerMap['organization'] as String?,
      timestamp: json['timestamp'] as String? ?? '',
      contentHash: json['signed_content_hash'] as String? ?? '',
    );
  }
}

/// Verification outcome for one signature (F22.S4).
class SignatureVerificationInfo {
  const SignatureVerificationInfo({
    required this.signatureId,
    required this.status,
    required this.signerName,
    required this.message,
  });

  final String signatureId;
  /// `valid` | `invalid` | `tampered`
  final String status;
  final String signerName;
  final String message;

  bool get isValid => status == 'valid';

  factory SignatureVerificationInfo.fromJson(Map<String, dynamic> json) {
    return SignatureVerificationInfo(
      signatureId: json['signature_id'] as String? ?? '',
      status: json['status'] as String? ?? 'invalid',
      signerName: json['signer_name'] as String? ?? '',
      message: json['message'] as String? ?? '',
    );
  }
}
