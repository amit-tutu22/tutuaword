/// Max page display-list snapshots kept off-screen (image/doc scroll RAM bound).
const kMaxCachedPageSnapshots = 32;

/// Returns page indices that should be dropped so [cachedPages] stays within
/// [maxCached], preferring pages farthest from [anchorPage].
List<int> pagesToEvictForLru({
  required Iterable<int> cachedPages,
  required int anchorPage,
  int maxCached = kMaxCachedPageSnapshots,
}) {
  final pages = cachedPages.toList();
  if (pages.length <= maxCached) return const [];
  pages.sort((a, b) {
    final da = (a - anchorPage).abs();
    final db = (b - anchorPage).abs();
    return db.compareTo(da);
  });
  final dropCount = pages.length - maxCached;
  return pages.take(dropCount).toList();
}
