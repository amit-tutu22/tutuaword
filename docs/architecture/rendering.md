# Rendering

The rendering subsystem (`tw-render`) converts layout output into a flat, GPU-ready display list that Flutter paints. Rust owns the entire pipeline from layout to display list; Flutter is a dumb painter.

## Why Not Flutter's Text Rendering

Flutter's `ui.Paragraph` uses its own text shaping (via SkParagraph/HarfBuzz internally). Using it would mean:

1. **Double shaping** — Rust shapes for layout, Flutter re-shapes for rendering. Results may differ.
2. **No glyph-level control** — Cannot render individual glyphs at Rust-computed positions.
3. **No complex script fidelity** — Flutter's shaping may differ from our rustybuzz pipeline for Arabic, Indic scripts, etc.
4. **Layout/render mismatch** — Cursor positioning and selection highlighting depend on exact glyph positions from the layout engine.

Instead, Rust shapes text, rasterizes glyphs into an atlas, and sends positioned glyph draws to Flutter via `canvas.drawRawAtlas()`.

## Rendering Pipeline

```
PageLayout (from tw-layout)
      │
      ▼
  DisplayListBuilder
  ├── Collect text lines → AtlasBatch
  ├── Collect borders/backgrounds → RectBatch
  ├── Collect shapes/lines → PathBatch
  └── Collect images → ImageBatch
      │
      ▼
  DisplayList (immutable snapshot)
      │
      ▼
  DisplayList::to_bytes() → flat binary
      │
      ▼
  FFI → Flutter CustomPainter
      │
      ▼
  canvas.drawRawAtlas() + drawRect() + drawPath() + drawImageRect()
```

## Display List Format (v3)

The display list is a single flat byte buffer — not a command array. After a fixed header and embedded glyph atlas pixels, four batches are appended in order: atlas (glyphs), rects, paths, images. There are no clip-stack commands; clipping is handled implicitly by batch bounds.

### Binary Layout

```
u32                     file_version (3)
u64                     snapshot version
f32                     page_width
f32                     page_height
u32                     atlas_width
u32                     atlas_height
u32                     atlas_pixel_len
u8[atlas_pixel_len]     RGBA atlas pixels

// AtlasBatch (glyphs)
u32                     glyph_count
f32[glyph_count * 2]    transforms ([x, y] per glyph)
f32[glyph_count * 4]    atlas rects ([atlasX, atlasY, atlasW, atlasH])
u32[glyph_count]        colors (ARGB tint)

// RectBatch
u32                     rect_count
f32[rect_count * 4]     rects ([x, y, w, h])
u32[rect_count]         colors (ARGB fill)

// PathBatch (v2+)
u32                     line_count
f32[line_count * 4]     line segments ([x1, y1, x2, y2])
u32[line_count]         colors

// ImageBatch (v2+; payloads added in v3)
u32                     image_count
f32[image_count * 2]    transforms
f32[image_count * 2]    sizes ([w, h])
image_count × { u32 len; u8[len] }   asset_id (UTF-8)
image_count × { u32 len; u8[len] }   payload (v3 only; empty in v2)
```

All multi-byte values are little-endian. Version 1 readers stop after the rect batch; version 2 adds path and image batches without payloads; version 3 adds encoded image payloads inline.

The in-memory struct mirrors the wire format:

```rust
pub struct DisplayList {
    pub version: u64,
    pub page_width: f32,
    pub page_height: f32,
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub atlas_pixels: Vec<u8>,
    pub atlas_batch: AtlasBatch,
    pub rect_batch: RectBatch,
    pub path_batch: PathBatch,
    pub image_batch: ImageBatch,
}
```

Serialization: `DisplayListBuilder::to_bytes(&list)`. Deserialization: `DisplayListBuilder::from_bytes(&bytes)`.

## Glyph Atlas

The glyph atlas is a shared texture containing pre-rasterized glyph bitmaps. All text on a page references into this atlas.

### Atlas Structure

```rust
pub struct GlyphAtlas {
    pub width: u32,          // texture width (default: 2048)
    pub height: u32,         // texture height (default: 2048)
    pub pixels: Vec<u8>,     // RGBA pixel data
    pub entries: HashMap<AtlasKey, AtlasEntry>,
    pub allocator: ShelfAllocator,
}

pub struct AtlasKey {
    pub font_id: FontId,
    pub glyph_id: u32,
    pub size: f32,           // font size in points
    pub subpixel: Subpixel,  // subpixel x/y offset for LCD rendering
}

pub struct AtlasEntry {
    pub x: u32,              // atlas x coordinate
    pub y: u32,              // atlas y coordinate
    pub width: u32,
    pub height: u32,
    pub bearing_x: f32,      // left offset from the pen position
    pub bearing_y: f32,      // height above the baseline
    pub is_color: bool,      // true for emoji/color bitmaps (never tinted)
}
```

### Pixel Format

Atlas pixels are premultiplied RGBA. Alpha-mask glyphs are stored as
premultiplied white (`rgb == a`) so the renderer can tint them with the run
color using a `Modulate` blend, which preserves antialiased edges. Color
bitmaps are stored as-is and are drawn with a white tint so `Modulate` leaves
them unchanged.

### Atlas Management

- **Shelf allocator** — glyphs packed in horizontal shelves, rows allocated top-to-bottom
- **Eviction** — when atlas is full, evict least-recently-used glyphs and re-rasterize on next frame
- **Subpixel positioning** — glyphs rasterized at 3× horizontal resolution for LCD subpixel rendering (desktop only; mobile uses standard grayscale)
- **Color glyphs** — emoji and color fonts rasterized as RGBA (not alpha-only)
- **Growth** — if atlas exceeds 2048×2048, allocate a second atlas page (multi-atlas support)

### Glyph Rasterization

Uses `swash` for glyph rasterization (`tw-shape/src/raster.rs`):

```rust
pub fn rasterize_glyph(
    &mut self,
    fonts: &FontDatabase,
    font_id: FontId,
    glyph_id: u32,
    size: f32,
) -> RasterizedGlyph {
    // Returns a premultiplied RGBA bitmap with placement bearings
}
```

Sources are tried in priority order — color outline, color bitmap, then
scalable outline — so emoji and CBDT/sbix strikes are handled alongside regular
text. Font file bytes are cached per `FontId`; layout consults the atlas first
and only calls into swash on a cache miss.

Rasterization happens on the worker thread during layout, not on the UI thread.

## Draw Batches

### AtlasBatch (Text Glyphs)

The primary text rendering command. Contains everything needed for `canvas.drawRawAtlas()`.

```rust
pub struct AtlasBatch {
    pub transforms: Vec<f32>,   // [x, y] top-left of each glyph quad (2 floats)
    pub rects: Vec<f32>,        // [atlasX, atlasY, atlasW, atlasH] per glyph (4 floats)
    pub colors: Vec<u32>,       // ARGB tint per glyph (1 u32)
}
```

The wire format stays compact; Flutter expands each `[x, y]` pair into the
4-float RST transform (`[scos, ssin, tx, ty]`) and each XYWH source rect into
the LTRB form that `drawRawAtlas` expects.

Flutter side:

```dart
canvas.drawRawAtlas(
  atlasImage,
  rstTransforms,       // Float32List: [scos, ssin, tx, ty] per glyph
  srcRects,            // Float32List: LTRB source rects in the atlas
  snapshot.glyphColors, // Int32List: ARGB tint per glyph
  ui.BlendMode.modulate,
  null,                // no cull rect
  Paint(),
);
```

`Modulate` multiplies the tint into the premultiplied mask. `srcOver` would
flood the entire sprite rectangle with the tint and render text as solid bars.

Each glyph is drawn as a separate entry in the batch. For a typical page with ~2000 glyphs, this is a single GPU draw call.

### RectBatch (Borders, Backgrounds, Highlights)

```rust
pub struct RectBatch {
    pub rects: Vec<f32>,   // [x, y, w, h] per rect
    pub colors: Vec<u32>,  // ARGB fill color
}
```

Used for:
- Paragraph background shading
- Text highlight color
- Table cell borders and backgrounds
- Page margins (debug overlay)
- Selection highlight
- Cursor (blinking rect)

### PathBatch (Table Grid Lines)

```rust
pub struct PathBatch {
    pub points: Vec<f32>,   // [x1, y1, x2, y2] per line segment
    pub colors: Vec<u32>,   // ARGB color per segment
}
```

Used for table grid lines. Each segment is a straight line between two points.

### ImageBatch (Embedded Images)

```rust
pub struct ImageBatch {
    pub transforms: Vec<f32>,   // [x, y] on page, per image
    pub sizes: Vec<f32>,        // [w, h] on page, per image
    pub asset_ids: Vec<String>, // package part name, e.g. word/media/image1.png
    pub payloads: Vec<Vec<u8>>, // encoded source bytes (PNG, JPEG, ...)
}
```

Image bytes travel inside the page's display list blob, the same way the glyph
atlas does, so there is no separate asset FFI to keep in sync. They stay in
their source encoding rather than being decoded to RGBA in Rust: Skia already
has the decoders, and a compressed logo is a fraction of the size of its raw
pixels.

Flutter decodes each payload once via `ui.ImageDescriptor.encoded` and caches
the resulting `ui.Image` under its asset id, keyed across pages so a repeated
asset (a header logo, say) is decoded a single time. `DocumentPainter` looks the
id up and calls `canvas.drawImageRect`; a miss — an unresolved relationship, or
a format Skia cannot read such as EMF/WMF — falls back to the grey placeholder
block that `DisplayListBuilder` emits for empty payloads.

Serialized layout, appended after the path batch (file version 3):

```
u32                     image_count
f32[image_count * 2]    transforms
f32[image_count * 2]    sizes
image_count × { u32 len; u8[len] }   asset_id (UTF-8)
image_count × { u32 len; u8[len] }   payload
```

Version 2 readers stop after the asset ids and see empty payloads.

## Flutter CustomPainter

```dart
class DocumentPainter extends CustomPainter {
  final DisplayListSnapshot snapshot;
  final ui.Image atlasImage;
  final Map<String, ui.Image> embeddedImages;

  @override
  void paint(Canvas canvas, Size size) {
    final list = snapshot.displayList;
    if (list.atlasBatch.transforms.isNotEmpty) {
      _paintAtlas(canvas, list.atlasBatch, atlasImage);
    }
    _paintRects(canvas, list.rectBatch);
    _paintPaths(canvas, list.pathBatch);
    _paintImages(canvas, list.imageBatch, embeddedImages);
  }

  @override
  bool shouldRepaint(DocumentPainter oldDelegate) {
    return snapshot.version != oldDelegate.snapshot.version;
  }
}
```

### Page Widget

Each page is a separate widget with its own `CustomPainter`:

```dart
class DocumentPage extends StatelessWidget {
  final int pageIndex;
  final DisplayListSnapshot snapshot;

  @override
  Widget build(BuildContext context) {
    return CustomPaint(
      painter: DocumentPainter(
        snapshot: snapshot,
        atlasImage: atlasImageFor(snapshot.atlasId),
        embeddedImages: embeddedImagesFor(pageIndex),
      ),
      size: Size(snapshot.pageWidth, snapshot.pageHeight),
    );
  }
}
```

Pages are laid out vertically in a scrollable viewport with configurable gap between pages.

## Overlay Rendering (Flutter-side)

Some elements are rendered by Flutter directly, not via the display list:

| Element | Why Flutter | Implementation |
|---------|-------------|----------------|
| Cursor (blinking caret) | Animation requires 60 FPS timer | Flutter `AnimationController` + rect draw |
| Selection highlight | Changes on every mouse drag event | Flutter rect draw from selection range |
| IME composition underline | Tied to Flutter's `TextInputConnection` | Flutter decoration |
| Live cursors (collaboration) | Colored per-user, animated | Flutter widget overlay |
| Rulers | Interactive (drag margins) | Flutter widget |
| Context menu | Native platform menu | Flutter `ContextMenuRegion` |

These overlays query the layout engine (via FFI) for position information but are painted by Flutter widgets on top of the document canvas.

## Zoom and Scroll

### Zoom

Zoom scales the canvas transform, not the layout:

```dart
Transform.scale(
  scale: zoomLevel,
  child: DocumentPage(pageIndex: i, snapshot: snapshots[i]),
)
```

Layout is computed at 1.0× (actual page dimensions). Zoom is purely a viewport transform. This avoids re-layout on every zoom change.

For zoom levels where text becomes unreadably small or large, re-layout at the target zoom level may be triggered (Phase 2+).

### Scroll

Standard Flutter `ScrollView` with page height × page count as total extent. Only visible pages (+ 1 page overscan) have active `CustomPainter` widgets.

## Performance

| Metric | Target | Strategy |
|--------|--------|----------|
| Paint single page | <3 ms | Single atlas draw call + batched rects |
| Paint during scroll (60 FPS) | <16 ms total frame | Only paint visible pages |
| Atlas upload (new page) | <5 ms | RGBA texture upload to GPU |
| Display list deserialization | <1 ms | Zero-copy from FFI buffer |
| Memory per page snapshot | ~500 KB | Atlas shared across pages |

### Display List Versioning

Each snapshot has a monotonically increasing version number. Flutter compares versions in `shouldRepaint()` to skip unnecessary repaints.

```rust
pub struct DisplayListSnapshot {
    pub version: u64,
    pub page_index: PageIndex,
    pub display_list: DisplayList,
    pub atlas: Arc<GlyphAtlas>,
}
```

When a page is not dirty, the previous snapshot is reused (no rebuild, no repaint).

## Debug Rendering

Development-only rendering modes (enabled via `--features debug-render`):

| Mode | Shows |
|------|-------|
| Layout boxes | Colored rectangles around every layout box |
| Line boundaries | Horizontal lines at line breaks |
| Baseline grid | Lines at every baseline |
| Float boundaries | Rectangles around floating objects |
| Atlas visualization | The glyph atlas texture |
| Invalidation flash | Highlight pages that were re-laid-out |

These are invaluable for layout debugging and are used in golden-image regression tests.
