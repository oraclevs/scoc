#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompatibilityBaseline {
    pub project: &'static str,
    pub version: &'static str,
    pub commit: &'static str,
}

pub const JC_BASELINE: CompatibilityBaseline = CompatibilityBaseline {
    project: "jc",
    version: "1.26.0",
    commit: "73fa7d5572dd730076723bd6280786bb9101d32f",
};
