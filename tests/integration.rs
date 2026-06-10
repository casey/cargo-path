use {
  regex::Regex,
  std::{fs, process::Command},
  tempfile::TempDir,
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

fn project(default: bool) -> TempDir {
  let dir = tempfile::tempdir().unwrap();

  let features = if default {
    "default = [\"baz\"]\nbaz = [\"dep:bar\"]"
  } else {
    "baz = [\"dep:bar\"]"
  };

  fs::write(
    dir.path().join("Cargo.toml"),
    format!(
      "[package]
name = \"foo\"
version = \"0.0.0\"
edition = \"2024\"

[dependencies]
bar = {{ path = \"bar\", optional = true }}

[features]
{features}
"
    ),
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

  dir
}

#[test]
fn feature_gated_dependency() {
  #[track_caller]
  fn case(default: bool, args: &[&str], found: bool) {
    let dir = project(default);

    let output = Command::new(env!("CARGO_BIN_EXE_cargo-path"))
      .arg("path")
      .args(args)
      .arg("bar")
      .current_dir(dir.path())
      .output()
      .unwrap();

    if found {
      assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr),
      );
      assert_eq!(
        str::from_utf8(&output.stdout).unwrap(),
        format!(
          "{}\n",
          dir.path().canonicalize().unwrap().join("bar").display(),
        ),
      );
    } else {
      assert!(!output.status.success());
      assert_eq!(
        str::from_utf8(&output.stderr).unwrap(),
        "error: dependency `bar` not found\n",
      );
    }
  }

  case(false, &[], true);
  case(false, &["--features", "baz"], true);
  case(false, &["--all-features"], true);
  case(true, &["--no-default-features"], false);
}
