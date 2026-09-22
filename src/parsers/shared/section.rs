#[derive(Clone, Debug)]
pub(crate) struct Section {
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum SectionMatcher {
    ColonHeader,
    BlankSeparated,
}

pub(crate) fn sections(lines: &[&str], matcher: SectionMatcher) -> Vec<Section> {
    let mut out = Vec::new();
    let mut current = Section {
        title: String::new(),
        lines: Vec::new(),
    };
    for line in lines {
        let trimmed = line.trim();
        let new_header = match matcher {
            SectionMatcher::ColonHeader => trimmed.ends_with(':') && !trimmed.contains("  "),
            SectionMatcher::BlankSeparated => false,
        };
        if new_header || (matches!(matcher, SectionMatcher::BlankSeparated) && trimmed.is_empty()) {
            if !current.title.is_empty() || !current.lines.is_empty() {
                out.push(current);
            }
            current = Section {
                title: trimmed.trim_end_matches(':').to_string(),
                lines: Vec::new(),
            };
        } else if !trimmed.is_empty() {
            if current.title.is_empty() {
                current.title = trimmed.to_string();
            } else {
                current.lines.push((*line).to_string());
            }
        }
    }
    if !current.title.is_empty() || !current.lines.is_empty() {
        out.push(current);
    }
    out
}
