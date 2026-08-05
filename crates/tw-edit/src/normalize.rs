use tw_model::Document;
use tw_text::TextBuffer;

pub fn normalize_runs(doc: &mut Document, buffer: &mut TextBuffer) {
    for para in doc.paragraphs_mut() {
        let mut i = 0;
        while i + 1 < para.runs.len() {
            let same_format = para.runs[i].format.equals(&para.runs[i + 1].format)
                && para.runs[i].revision == para.runs[i + 1].revision;
            if same_format {
                let suffix_id = para.runs[i + 1].id;
                let suffix_text = buffer.to_string(suffix_id);
                let run_id = para.runs[i].id;
                if let Some(a) = para.runs[i].text_mut() {
                    a.push_str(&suffix_text);
                    let merged = a.clone();
                    buffer.sync_from_run(run_id, &merged);
                }
                buffer.unregister(suffix_id);
                para.runs.remove(i + 1);
            } else {
                i += 1;
            }
        }

        let mut to_remove = Vec::new();
        for run in &para.runs {
            if run.text().is_empty() && para.runs.len() > 1 {
                to_remove.push(run.id);
            }
        }
        for id in to_remove {
            buffer.unregister(id);
            para.runs.retain(|r| r.id != id);
        }

        if para.runs.is_empty() {
            let run = tw_model::Run::new_text("");
            buffer.register(run.id, "");
            para.runs.push(run);
        }
    }
}
