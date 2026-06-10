use super::*;

#[derive(Parser)]
#[command(
  name = "cargo",
  bin_name = "cargo",
  styles = styling::Styles::styled()
    .header(styling::AnsiColor::Yellow.on_default().bold())
    .usage(styling::AnsiColor::Yellow.on_default().bold())
    .literal(styling::AnsiColor::Green.on_default().bold())
    .placeholder(styling::AnsiColor::Cyan.on_default()),
)]
pub(crate) enum Arguments {
  #[command(name = "path")]
  Path(Path),
}
