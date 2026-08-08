use tw_model::{Document, NodeId, RunLocation};
use tw_model::Run;

pub(crate) fn with_run_mut<F, R>(doc: &mut Document, run_id: NodeId, f: F) -> Option<R>
where
    F: FnOnce(&mut Run) -> R,
{
    let loc = doc.find_run_location(run_id)?;
    let para = doc.paragraph_at_loc_mut(loc)?;
    let run = para.runs.get_mut(loc.run_index)?;
    Some(f(run))
}

pub(crate) fn with_paragraph_mut<F, R>(doc: &mut Document, para_id: NodeId, f: F) -> Option<R>
where
    F: FnOnce(&mut tw_model::Paragraph) -> R,
{
    let loc = doc.find_paragraph_run_location(para_id)?;
    let para = doc.paragraph_at_loc_mut(loc)?;
    Some(f(para))
}

pub(crate) fn run_location(doc: &Document, run_id: NodeId) -> Option<RunLocation> {
    doc.find_run_location(run_id)
}
