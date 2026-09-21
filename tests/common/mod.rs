use std::path::PathBuf;

pub fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("jc-1.26.0")
        .join(relative)
}

pub fn fixture(relative: &str) -> Vec<u8> {
    let path = fixture_path(relative);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("could not read fixture {}: {error}", path.display()))
}
