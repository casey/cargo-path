use {
  regex::Regex,
  std::{fs, path::Path, process::Command},
};

const REGEX_VERSION: &str = "1.12.3";

#[track_caller]
fn case(dir: &Path, dependency: &str, found: bool) {
  let output = Command::new(env!("CARGO_BIN_EXE_cargo-path"))
    .args(["path", dependency])
    .current_dir(dir)
    .output()
    .unwrap();

  if found {
    assert!(
      output.status.success(),
      "{}",
      String::from_utf8_lossy(&output.stderr),
    );

    let stdout = str::from_utf8(&output.stdout).unwrap();

    let regex = Regex::new(&format!(
      r"^.*/\.cargo/registry/src/index\.crates\.io-[0-9a-f]*/regex-{}\n$",
      regex::escape(REGEX_VERSION),
    ))
    .unwrap();

    assert!(
      regex.is_match(stdout),
      "regex mismatch: {stdout} ~= {regex}",
    );
  } else {
    assert!(!output.status.success());
    assert_eq!(
      str::from_utf8(&output.stderr).unwrap(),
      format!("error: dependency `{dependency}` not found\n"),
    );
  }
}

#[test]
fn single_dependency() {
  case(Path::new("."), "regex", true);
}

#[test]
fn feature_gated_dependency() {
  let dir = tempfile::tempdir().unwrap();

  fs::write(
    dir.path().join("Cargo.toml"),
    format!(
      "[package]
name = \"foo\"
version = \"0.0.0\"
edition = \"2024\"

[dependencies]
regex = {{ version = \"={REGEX_VERSION}\", optional = true }}

[features]
bar = [\"dep:regex\"]
"
    ),
  )
  .unwrap();

  fs::create_dir(dir.path().join("src")).unwrap();
  fs::write(dir.path().join("src/lib.rs"), "").unwrap();

  case(dir.path(), "regex", true);
  case(dir.path(), "qux", false);
}
