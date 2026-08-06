//! F02.S1 integration gate: insert + layout p99 under 15 ms (release builds).

use std::time::Instant;

use tw_core::SyncSession;
use tw_edit::Command;

const SAMPLE_COUNT: usize = 200;
const P99_GATE_MS: f64 = 15.0;

fn caret_run_and_offset(session: &SyncSession) -> (tw_model::NodeId, usize) {
    let para = session.edit.document.paragraph_at(0, 0).unwrap();
    let run = para.runs.last().expect("paragraph has a run");
    (run.id, run.text().len())
}

fn measure_keystroke_ms(session: &mut SyncSession) -> f64 {
    let (run_id, offset) = caret_run_and_offset(session);
    let start = Instant::now();
    session.apply(Command::InsertText {
        run_id,
        offset,
        text: "a".into(),
    });
    let _ = session.display_list_bytes();
    start.elapsed().as_secs_f64() * 1000.0
}

fn percentile(sorted: &[f64], pct: f64) -> f64 {
    let idx = ((sorted.len() as f64 * pct).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len() - 1);
    sorted[idx]
}

#[test]
fn i_f02_s1_typing_latency_p99_under_15ms() {
    if cfg!(debug_assertions) {
        // Debug builds are too slow for the performance gate; CI runs this in release.
        let mut session = SyncSession::new();
        for _ in 0..20 {
            let _ = measure_keystroke_ms(&mut session);
        }
        return;
    }

    let mut session = SyncSession::new();
    for _ in 0..20 {
        let _ = measure_keystroke_ms(&mut session);
    }

    let mut samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        samples.push(measure_keystroke_ms(&mut session));
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let p99 = percentile(&samples, 0.99);
    assert!(
        p99 < P99_GATE_MS,
        "p99 typing latency {:.2} ms exceeds {:.0} ms gate (p50 {:.2} ms)",
        p99,
        P99_GATE_MS,
        percentile(&samples, 0.50)
    );
}
