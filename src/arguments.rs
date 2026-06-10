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
    #[arg(long, help = "Activate all available features")]
    all_features: bool,
    #[arg(
      short = 'F',
      long,
      value_delimiter = ',',
      value_name = "FEATURES",
      help = "Comma separated list of features to activate"
    )]
    features: Vec<String>,
    #[arg(long, help = "Do not activate the `default` feature")]
    no_default_features: bool,
    #[arg(help = "Dependency name")]
    dependency: String,
  },
}

impl Arguments {
  pub(crate) fn run(self) -> Result<(), Error> {
    let Self::Path {
      all_features,
      features,
      no_default_features,
      dependency,
    } = &self;

    let metadata = self.metadata(*all_features)?;

    let paths = Self::search(&metadata, dependency)?;

    let paths = if paths.is_empty() && !all_features && features.is_empty() && !no_default_features
    {
      let metadata = self.metadata(true)?;
      Self::search(&metadata, dependency)?
    } else {
      paths
    };

    ensure!(
      !paths.is_empty(),
      error::DependencyNotFound {
        dependency: dependency.as_str(),
      }
    );

    for path in &paths {
      println!("{path}");
    }

    Ok(())
  }

  fn metadata(&self, all_features: bool) -> Result<Metadata, Error> {
    let Self::Path {
      features,
      no_default_features,
      ..
    } = self;

    let mut command = MetadataCommand::new();

    if all_features {
      command.features(CargoOpt::AllFeatures);
    }

    if *no_default_features {
      command.features(CargoOpt::NoDefaultFeatures);
    }

    if !features.is_empty() {
      command.features(CargoOpt::SomeFeatures(features.clone()));
    }

    command.exec().context(error::Metadata)
  }

  fn search(metadata: &Metadata, dependency: &str) -> Result<Vec<Utf8PathBuf>, Error> {
    let resolve = metadata.resolve.as_ref().context(error::MissingResolve)?;

    let root = resolve.root.as_ref().context(error::MissingRoot)?;

    let nodes = resolve
      .nodes
      .iter()
      .map(|node| (&node.id, node.dependencies.as_slice()))
      .collect::<HashMap<&PackageId, &[PackageId]>>();

    let mut depths = HashMap::<&PackageId, u32>::new();
    let mut queue = VecDeque::new();

    depths.insert(root, 0);
    queue.push_back(root);

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
