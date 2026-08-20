import 'package:flutter_test/flutter_test.dart';
import 'package:tutuaword/bridge/external_link.dart';

void main() {
  group('document link scheme allowlist', () {
    test('allows http https mailto tel', () {
      expect(isAllowedDocumentLinkUri(Uri.parse('https://example.com/a')), isTrue);
      expect(isAllowedDocumentLinkUri(Uri.parse('http://example.com')), isTrue);
      expect(isAllowedDocumentLinkUri(Uri.parse('mailto:user@example.com')), isTrue);
      expect(isAllowedDocumentLinkUri(Uri.parse('tel:+15551212')), isTrue);
    });

    test('blocks javascript file data and custom schemes', () {
      expect(isAllowedDocumentLinkUri(Uri.parse('javascript:alert(1)')), isFalse);
      expect(isAllowedDocumentLinkUri(Uri.parse('file:///etc/passwd')), isFalse);
      expect(isAllowedDocumentLinkUri(Uri.parse('data:text/html,hi')), isFalse);
      expect(isAllowedDocumentLinkUri(Uri.parse('intent://scan/#Intent;end')), isFalse);
      expect(isAllowedDocumentLinkUri(Uri.parse('mybank://pay')), isFalse);
      expect(isAllowedDocumentLinkUri(Uri.parse('ftp://example.com')), isFalse);
    });

    test('requires host for http(s)', () {
      expect(isAllowedDocumentLinkUri(Uri.parse('https://')), isFalse);
      expect(isAllowedDocumentLinkUri(Uri.parse('http:')), isFalse);
    });
  });
}
