/// Built-in starter document templates (F24.S1 / F24.S2).
class DocumentTemplateSpec {
  const DocumentTemplateSpec({
    required this.id,
    required this.title,
    required this.subtitle,
    required this.assetPath,
    required this.markerText,
    required this.themeName,
  });

  final String id;
  final String title;
  final String subtitle;
  /// Path under Flutter assets (declared in pubspec.yaml).
  final String assetPath;
  /// Distinctive phrase present in the template body (for tests / status).
  final String markerText;
  /// Design gallery theme applied when opening this template (F24.S2).
  /// One of: Office, Facet, Ion.
  final String themeName;

  /// Slug for user-template filenames / ids (F24.S3).
  static String slugifyTitle(String title) {
    final slug = title
        .toLowerCase()
        .trim()
        .replaceAll(RegExp(r'[^a-z0-9]+'), '-')
        .replaceAll(RegExp(r'^-+|-+$'), '');
    return slug.isEmpty ? 'template' : slug;
  }

  static const resume = DocumentTemplateSpec(
    id: 'resume',
    title: 'Resume',
    subtitle: 'Professional CV with experience sections',
    assetPath: 'assets/templates/resume.docx',
    markerText: 'Professional Resume',
    themeName: 'Facet',
  );

  static const letter = DocumentTemplateSpec(
    id: 'letter',
    title: 'Letter',
    subtitle: 'Formal business letter',
    assetPath: 'assets/templates/letter.docx',
    markerText: 'Business Letter',
    themeName: 'Office',
  );

  static const invoice = DocumentTemplateSpec(
    id: 'invoice',
    title: 'Invoice',
    subtitle: 'Simple itemized invoice table',
    assetPath: 'assets/templates/invoice.docx',
    markerText: 'Invoice',
    themeName: 'Ion',
  );

  static const brochure = DocumentTemplateSpec(
    id: 'brochure',
    title: 'Brochure',
    subtitle: 'Product one-pager',
    assetPath: 'assets/templates/brochure.docx',
    markerText: 'Product Brochure',
    themeName: 'Facet',
  );

  static const newsletter = DocumentTemplateSpec(
    id: 'newsletter',
    title: 'Newsletter',
    subtitle: 'Team update layout',
    assetPath: 'assets/templates/newsletter.docx',
    markerText: 'Team Newsletter',
    themeName: 'Ion',
  );

  static const businessProposal = DocumentTemplateSpec(
    id: 'business_proposal',
    title: 'Business Proposal',
    subtitle: 'Client proposal outline',
    assetPath: 'assets/templates/business_proposal.docx',
    markerText: 'Business Proposal',
    themeName: 'Office',
  );

  static const researchPaper = DocumentTemplateSpec(
    id: 'research_paper',
    title: 'Research Paper',
    subtitle: 'Academic paper sections',
    assetPath: 'assets/templates/research_paper.docx',
    markerText: 'Research Paper',
    themeName: 'Facet',
  );

  static const all = <DocumentTemplateSpec>[
    resume,
    letter,
    invoice,
    brochure,
    newsletter,
    businessProposal,
    researchPaper,
  ];

  /// Gallery theme names used by the Design tab / `DocumentTheme::by_name`.
  static const galleryThemeNames = <String>['Office', 'Facet', 'Ion'];

  static DocumentTemplateSpec? byId(String id) {
    for (final t in all) {
      if (t.id == id) return t;
    }
    return null;
  }
}

/// User-saved template metadata under `~/.tutuaword/templates/` (F24.S3).
class UserTemplateEntry {
  const UserTemplateEntry({
    required this.id,
    required this.title,
    required this.themeName,
    required this.fileName,
    required this.createdAt,
  });

  final String id;
  final String title;
  /// Snapshot of [EditorController.documentThemeName] at save time.
  final String themeName;
  final String fileName;
  final DateTime createdAt;

  factory UserTemplateEntry.fromJson(Map<String, dynamic> json) {
    return UserTemplateEntry(
      id: json['id'] as String,
      title: json['title'] as String,
      themeName: json['themeName'] as String? ?? 'Office',
      fileName: json['fileName'] as String,
      createdAt: DateTime.parse(json['createdAt'] as String),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'title': title,
      'themeName': themeName,
      'fileName': fileName,
      'createdAt': createdAt.toUtc().toIso8601String(),
    };
  }
}

/// Chooser result for New from Template (builtin pack or My Templates).
class TemplateSelection {
  const TemplateSelection.builtin(this.spec) : user = null;
  const TemplateSelection.user(this.user) : spec = null;

  final DocumentTemplateSpec? spec;
  final UserTemplateEntry? user;

  bool get isBuiltin => spec != null;
  String get title => spec?.title ?? user!.title;
  String get themeName => spec?.themeName ?? user!.themeName;
}
