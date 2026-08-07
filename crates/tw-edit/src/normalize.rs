use tw_model::Document;

pub fn normalize_runs(doc: &mut Document) {
    for para in doc.paragraphs_mut() {
        let mut i = 0;
        while i + 1 < para.runs.len() {
            let same_format = para.runs[i].format.merge_equivalent(&para.runs[i + 1].format)
                && para.runs[i].revision == para.runs[i + 1].revision
                && para.runs[i].content.is_mergeable_text()
                && para.runs[i + 1].content.is_mergeable_text();
            if same_format {
                let suffix_text = para.runs[i + 1].text().to_string();
                if let Some(a) = para.runs[i].text_mut() {
                    a.push_str(&suffix_text);
                }
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
            para.runs.retain(|r| r.id != id);
        }

        if para.runs.is_empty() {
            para.runs.push(tw_model::Run::new_text(""));
        }
    }
}
