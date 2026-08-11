/// Document position mirroring Rust `tw_edit::DocPosition`.
class DocPosition {
  const DocPosition({required this.runId, required this.offset});

  final String runId;
  final int offset;

  @override
  bool operator ==(Object other) =>
      other is DocPosition && other.runId == runId && other.offset == offset;

  @override
  int get hashCode => Object.hash(runId, offset);

  DocPosition copyWith({String? runId, int? offset}) =>
      DocPosition(runId: runId ?? this.runId, offset: offset ?? this.offset);
}

/// Document range mirroring Rust `tw_edit::DocRange` (anchor + focus).
class DocRange {
  const DocRange({
    required this.anchor,
    required this.focus,
    this.page = 0,
  });

  final DocPosition anchor;
  final DocPosition focus;
  final int page;

  bool get isCollapsed =>
      anchor.runId == focus.runId && anchor.offset == focus.offset;

  bool get isValid => anchor.runId.isNotEmpty && focus.runId.isNotEmpty;

  /// Normalized start/end for engine commands (start ≤ end in document order).
  (DocPosition start, DocPosition end) normalized() {
    if (anchor.runId == focus.runId) {
      if (anchor.offset <= focus.offset) {
        return (anchor, focus);
      }
      return (focus, anchor);
    }
    // Cross-run: preserve anchor→focus document order (engine handles ordering).
    return (anchor, focus);
  }

  DocRange copyWith({
    DocPosition? anchor,
    DocPosition? focus,
    int? page,
  }) =>
      DocRange(
        anchor: anchor ?? this.anchor,
        focus: focus ?? this.focus,
        page: page ?? this.page,
      );

  @override
  bool operator ==(Object other) =>
      other is DocRange &&
      other.anchor == anchor &&
      other.focus == focus &&
      other.page == page;

  @override
  int get hashCode => Object.hash(anchor, focus, page);
}
