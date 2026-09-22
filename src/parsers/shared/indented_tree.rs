#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TreeRow {
    pub depth: usize,
    pub text: String,
}

pub(crate) fn indented_tree(lines: &[&str]) -> Vec<TreeRow> {
    lines
        .iter()
        .filter_map(|line| {
            if line.trim().is_empty() {
                return None;
            }
            let prefix = line
                .chars()
                .take_while(|c| {
                    c.is_whitespace()
                        || matches!(*c, '│' | '├' | '└' | '─' | '+' | '\\' | '|' | '`')
                })
                .collect::<String>();
            let depth = prefix
                .chars()
                .filter(|c| matches!(*c, '│' | '├' | '└' | '+' | '\\' | '|' | '`'))
                .count()
                .max(prefix.chars().filter(|c| *c == ' ').count() / 2);
            let text = line
                .trim_start_matches(|c: char| {
                    c.is_whitespace() || matches!(c, '│' | '├' | '└' | '─' | '+' | '\\' | '|' | '`')
                })
                .trim()
                .to_string();
            Some(TreeRow { depth, text })
        })
        .collect()
}
