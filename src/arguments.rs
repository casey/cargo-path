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
  Path {
    #[arg(help = "Dependency name")]
    dependency: String,
  },
}

impl Arguments {
  pub(crate) fn run(self) -> Result<(), Error> {
    let Self::Path { dependency } = self;

    let metadata = MetadataCommand::new().exec().context(error::Metadata)?;

    let paths = Self::search(&metadata, &dependency)?;

    let paths = if paths.is_empty() {
      let metadata = MetadataCommand::new()
        .features(CargoOpt::AllFeatures)
        .exec()
        .context(error::Metadata)?;
      Self::search(&metadata, &dependency)?
    } else {
      paths
    };

    ensure!(!paths.is_empty(), error::DependencyNotFound { dependency });

    for path in &paths {
      println!("{path}");
    }

    Ok(())
  }

  fn search(metadata: &Metadata, dependency: &str) -> Result<Vec<Utf8PathBuf>, Error> {
    let resolve = metadata.resolve.as_ref().context(error::MissingResolve)?;

    let roots = if let Some(root) = resolve.root.as_ref() {
      vec![root]
    } else {
      metadata
        .workspace_members
        .iter()
        .collect::<Vec<&PackageId>>()
    };

    ensure!(!roots.is_empty(), error::MissingRoots);

    let nodes = resolve
      .nodes
      .iter()
      .map(|node| (&node.id, node.dependencies.as_slice()))
      .collect::<HashMap<&PackageId, &[PackageId]>>();

    let mut depths = HashMap::<&PackageId, u32>::with_capacity(roots.len());
    let mut queue = VecDeque::with_capacity(roots.len());

    for root in roots {
      depths.insert(root, 0);
      queue.push_back(root);
    }

    while let Some(id) = queue.pop_front() {
      let depth = depths[id];
      if let Some(deps) = nodes.get(id) {
        for dep in *deps {
          if !depths.contains_key(dep) {
            depths.insert(dep, depth + 1);
            queue.push_back(dep);
          }
        }
      }
    }

    let mut matches = metadata
      .packages
      .iter()
      .filter(|p| p.name.as_str() == dependency)
      .filter_map(|p| {
        Some((
          depths.get(&p.id).copied()?,
          &p.version,
          p.manifest_path.parent().unwrap(),
        ))
      })
      .collect::<Vec<(u32, &Version, &Utf8Path)>>();

    matches.sort();

    Ok(
      matches
        .into_iter()
        .map(|(_depth, _version, path)| path.to_owned())
        .collect(),
    )
  }
}
