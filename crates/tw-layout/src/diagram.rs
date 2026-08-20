//! SmartArt placeholder node geometry shared by layout and render.

use tw_model::DiagramKind;

/// Axis-aligned node frames `[x, y, width, height]` for an inserted SmartArt preview.
pub fn diagram_node_rects(
    kind: DiagramKind,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Vec<[f32; 4]> {
    match kind {
        DiagramKind::Hierarchy => hierarchy_node_rects(x, y, width, height),
        DiagramKind::Cycle => cycle_node_rects(x, y, width, height),
        DiagramKind::Process => process_node_rects(x, y, width, height),
    }
}

fn process_node_rects(x: f32, y: f32, width: f32, height: f32) -> Vec<[f32; 4]> {
    let pad_x = width * 0.08;
    let pad_top = height * 0.28;
    let pad_bottom = height * 0.18;
    let body_h = (height - pad_top - pad_bottom).max(24.0);
    let gap = width * 0.06;
    let node_w = ((width - pad_x * 2.0 - gap * 2.0) / 3.0).max(28.0);
    let node_h = body_h.min(height * 0.42).max(20.0);
    let ny = y + pad_top + (body_h - node_h) * 0.5;
    (0..3)
        .map(|i| {
            let nx = x + pad_x + i as f32 * (node_w + gap);
            [nx, ny, node_w, node_h]
        })
        .collect()
}

fn hierarchy_node_rects(x: f32, y: f32, width: f32, height: f32) -> Vec<[f32; 4]> {
    let top_w = width * 0.28;
    let top_h = height * 0.16;
    let top_x = x + (width - top_w) * 0.5;
    let top_y = y + height * 0.28;

    let mid_y = top_y + top_h + 12.0;
    let child_w = width * 0.22;
    let child_h = height * 0.16;
    let gap = width * 0.06;
    let row_w = child_w * 3.0 + gap * 2.0;
    let row_x = x + (width - row_w) * 0.5;

    let mut out = vec![[top_x, top_y, top_w, top_h]];
    for i in 0..3 {
        let cx = row_x + i as f32 * (child_w + gap);
        out.push([cx, mid_y + 8.0, child_w, child_h]);
    }
    out
}

fn cycle_node_rects(x: f32, y: f32, width: f32, height: f32) -> Vec<[f32; 4]> {
    let cx = x + width * 0.5;
    let cy = y + height * 0.55;
    let radius = (width.min(height) * 0.22).max(28.0);
    (0..3)
        .map(|i| {
            let t = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * i as f32 / 3.0;
            let nx = cx + radius * t.cos() - 18.0;
            let ny = cy + radius * t.sin() - 12.0;
            [nx, ny, 36.0, 24.0]
        })
        .collect()
}
