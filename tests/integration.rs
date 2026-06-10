use {
  regex::Regex,
  std::{fs, path::Path, process::Command},
};

#[track_caller]
fn case(dir: &Path, dependency: &str, found: bool) {
  let output = Command::new(env!("CARGO_BIN_EXE_cargo-path"))
    .args(["path", dependency])
    .current_dir(dir)
    .output()
    .unwrap();

  let stderr = str::from_utf8(&output.stderr).unwrap();

  if found {
    assert!(output.status.success(), "{stderr}");

    let stdout = str::from_utf8(&output.stdout).unwrap();

    let regex =
      Regex::new(r"^.*/\.cargo/registry/src/index\.crates\.io-[0-9a-f]*/regex-[0-9.]+\n$").unwrap();

    assert!(
      regex.is_match(stdout),
      "regex mismatch: {stdout} ~= {regex}",
    );
  } else {
    assert!(!output.status.success());
    assert_eq!(stderr, format!("error: dependency `{dependency}` not found\n"));
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
    "[package]
name = \"foo\"
version = \"0.0.0\"
edition = \"2024\"

[dependencies]
regex = { version = \"1.0.0\", optional = true }

[features]
bar = [\"dep:regex\"]
",
  )
  .unwrap();

  fs::create_dir(dir.path().join("src")).unwrap();
  fs::write(dir.path().join("src/lib.rs"), "").unwrap();

  case(dir.path(), "regex", true);
  case(dir.path(), "qux", false);
}
