use {
  regex::Regex,
  std::{fs, process::Command},
};

#[test]
fn single_dependency() {
  let output = Command::new(env!("CARGO_BIN_EXE_cargo-path"))
    .args(["path", "serde"])
    .output()
    .unwrap();

  assert!(output.status.success());

  let stdout = str::from_utf8(&output.stdout).unwrap();

  let regex =
    Regex::new(r"^.*/\.cargo/registry/src/index\.crates\.io-[0-9a-f]*/serde-1\.0\.228\n$").unwrap();

  assert!(
    regex.is_match(stdout),
    "regex mismatch: {stdout} ~= {regex}",
  );
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
bar = { path = \"bar\", optional = true }

[features]
baz = [\"dep:bar\"]
",
  )
  .unwrap();

  fs::create_dir(dir.path().join("src")).unwrap();
  fs::write(dir.path().join("src/lib.rs"), "").unwrap();

  fs::create_dir_all(dir.path().join("bar/src")).unwrap();
  fs::write(
    dir.path().join("bar/Cargo.toml"),
    "[package]
name = \"bar\"
version = \"0.0.0\"
edition = \"2024\"
",
  )
  .unwrap();
  fs::write(dir.path().join("bar/src/lib.rs"), "").unwrap();

  #[track_caller]
  fn case(dir: &std::path::Path, dependency: &str, expected: Option<&str>) {
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-path"))
      .args(["path", dependency])
      .current_dir(dir)
      .output()
      .unwrap();

    if let Some(expected) = expected {
      assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr),
      );
      assert_eq!(
        str::from_utf8(&output.stdout).unwrap(),
        format!("{}\n", dir.canonicalize().unwrap().join(expected).display(),),
      );
    } else {
      assert!(!output.status.success());
      assert_eq!(
        str::from_utf8(&output.stderr).unwrap(),
        format!("error: dependency `{dependency}` not found\n"),
      );
    }
  }

  case(dir.path(), "bar", Some("bar"));
  case(dir.path(), "qux", None);
}
