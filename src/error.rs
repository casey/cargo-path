use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum Error {
  #[snafu(display("dependency `{dependency}` not found"))]
  DependencyNotFound { dependency: String },
  #[snafu(display("failed to run `cargo metadata`"))]
  Metadata { source: cargo_metadata::Error },
  #[snafu(display("missing resolve in cargo metadata"))]
  MissingResolve,
  #[snafu(display("missing root packages in cargo metadata"))]
  MissingRoots,
}
