use tw_model::Document;
use tw_model::NodeId;

pub(crate) fn with_run_mut<F, R>(doc: &mut Document, run_id: NodeId, f: F) -> Option<R>
where
    F: FnOnce(&mut tw_model::Run) -> R,
{
    let (si, bi, ri) = doc.find_run_location(run_id)?;
    let para = doc.paragraph_at_mut(si, bi)?;
    let run = para.runs.get_mut(ri)?;
    Some(f(run))
}

pub(crate) fn with_paragraph_mut<F, R>(doc: &mut Document, para_id: NodeId, f: F) -> Option<R>
where
    F: FnOnce(&mut tw_model::Paragraph) -> R,
{
    let (si, bi) = doc.find_paragraph_location(para_id)?;
    let para = doc.paragraph_at_mut(si, bi)?;
    Some(f(para))
}
