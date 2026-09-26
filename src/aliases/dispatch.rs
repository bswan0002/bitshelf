use super::*;
use crate::config::Config;

pub(super) struct Plan {
    executable: OsString,
    argv: Vec<OsString>,
    command: Option<String>,
    environment: BTreeMap<String, OsString>,
}
impl Plan {
    fn execute(&self, name: &str) -> Result<()> {
        let mut command = std::process::Command::new(&self.executable);
        command.args(&self.argv).envs(&self.environment);
        if self.command.is_some() {
            command.env_remove("BASH_ENV");
        }
        let status = command
            .status()
            .with_context(|| format!("cannot execute helper alias {name} or extension"))?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn plan(name: &str, args: &[OsString], cfg: &Config, path: &Path) -> Result<Plan> {
    let executable = std::env::current_exe()?;
    let environment = BTreeMap::from([
        ("BS_CONFIG".into(), path.as_os_str().to_owned()),
        ("BS_EXECUTABLE".into(), executable.as_os_str().to_owned()),
    ]);
    let mut p = Plan {
        executable: executable.into_os_string(),
        argv: vec![],
        command: None,
        environment,
    };
    let Some(alias) = cfg.aliases.get(name) else {
        p.executable = discovery::executables()
            .remove(name)
            .with_context(|| format!("unknown extension {name}"))?
            .into_os_string();
        p.argv = args.to_vec();
        return Ok(p);
    };
    let d = alias.definition();
    if d.run.is_some() {
        let rendered = recipes::render(&d, args, cfg, path)?;
        let command = format!("bs() {{ \"$BS_EXECUTABLE\" \"$@\"; }}\n{rendered}");
        // Bash pipefail makes an upstream failure visible; SIGPIPE is also a failure.
        p.executable = "/bin/bash".into();
        p.argv = vec![
            "--noprofile".into(),
            "--norc".into(),
            "-e".into(),
            "-o".into(),
            "pipefail".into(),
            "-c".into(),
            command.clone().into(),
        ];
        p.command = Some(command);
    } else if let Some(exec) = d.exec {
        p.executable = exec[0].clone().into();
        p.argv = exec[1..]
            .iter()
            .map(OsString::from)
            .chain(args.iter().cloned())
            .collect();
    } else {
        let template = d.argv.unwrap();
        if template.iter().any(|s| s.contains('{')) {
            let mut positional = Vec::new();
            let mut forwarded = Vec::new();
            let mut literal = false;
            for arg in args {
                if !literal && arg == "--" {
                    literal = true;
                } else if !literal && arg.to_string_lossy().starts_with('-') {
                    crate::usage_check(
                        arg == "--json" || arg == "--dry-run",
                        "ID aliases accept only --json, --dry-run and --config; configure other arguments in the alias",
                    )?;
                    forwarded.push(arg.clone());
                } else {
                    positional.push(arg);
                }
            }
            crate::usage_check(
                positional.len() == 1,
                &format!("bs {name} requires exactly one shelf/bit-name ID"),
            )?;
            let id = positional[0].to_str().context("alias ID must be UTF-8")?;
            let parts: Vec<_> = id.split('/').collect();
            ensure!(parts.len() == 2, "expected shelf/bit-name identifier");
            for part in parts {
                crate::config::name(part)?;
            }
            p.argv = expand_template(&template, id, &forwarded)?;
        } else {
            p.argv = template
                .iter()
                .map(OsString::from)
                .chain(args.iter().cloned())
                .collect();
        }
    }
    Ok(p)
}

/// Discovery works before initialization; recipes still require configured definitions.
pub fn load_config(path: &Path) -> Result<Config> {
    if path.try_exists()? {
        Config::load(path)
    } else {
        Ok(Config {
            store: PathBuf::new(),
            editor: None,
            aliases: BTreeMap::new(),
        })
    }
}

pub fn inspect(cfg: &Config, path: &Path, args: &crate::cli::Aliases, json: bool) -> Result<()> {
    let catalog = discovery::catalog(cfg, path)?;
    match args.action.as_deref() {
        None => {
            crate::usage_check(
                args.name.is_none() && args.args.is_empty(),
                "use aliases show NAME or aliases dry-run NAME -- ARGS",
            )?;
            let human = catalog
                .values()
                .map(|e| {
                    format!(
                        "{}\t{}\t{}",
                        e.name,
                        e.kind,
                        e.description
                            .as_deref()
                            .unwrap_or("(no description provided)")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            crate::output::emit(&catalog, json, human)
        }
        Some("show") => {
            crate::usage_check(
                args.args.is_empty(),
                "show does not accept forwarded arguments",
            )?;
            let name = args
                .name
                .as_deref()
                .ok_or_else(|| crate::UsageError("aliases show requires a name".into()))?;
            let entry = catalog
                .get(name)
                .with_context(|| format!("unknown extension {name}"))?;
            crate::output::emit(entry, json, serde_json::to_string_pretty(entry)?)
        }
        Some("dry-run") => {
            let name = args
                .name
                .as_deref()
                .ok_or_else(|| crate::UsageError("aliases dry-run requires a name".into()))?;
            let forwarded: Vec<_> = args.args.iter().map(OsString::from).collect();
            let (selected, forwarded) = extract_config(&forwarded, Some(path.to_owned()))?;
            let path = crate::config::config_path(selected.as_deref())?;
            let cfg = load_config(&path)?;
            let p = plan(name, &forwarded, &cfg, &path)?;
            // JSON encodes argv as text, rather than platform-specific OsString representations.
            let text = |s: &OsString| {
                s.clone()
                    .into_string()
                    .map_err(|_| anyhow::anyhow!("preview requires UTF-8 paths and arguments"))
            };
            let argv = p.argv.iter().map(text).collect::<Result<Vec<_>>>()?;
            let env = p
                .environment
                .iter()
                .map(|(k, v)| Ok((k, text(v)?)))
                .collect::<Result<BTreeMap<_, _>>>()?;
            let unset_environment: Vec<&str> = if p.command.is_some() {
                vec!["BASH_ENV"]
            } else {
                vec![]
            };
            let value = serde_json::json!({"executable": text(&p.executable)?, "argv": argv, "command": p.command, "environment": env, "unset_environment": unset_environment});
            crate::output::emit(&value, json, serde_json::to_string_pretty(&value)?)
        }
        _ => unreachable!(),
    }
}

fn extract_config(
    args: &[OsString],
    mut config: Option<PathBuf>,
) -> Result<(Option<PathBuf>, Vec<OsString>)> {
    let mut out = Vec::new();
    let mut iter = args.iter();
    let mut literal = false;
    while let Some(arg) = iter.next() {
        if !literal && arg == "--config" {
            config = Some(PathBuf::from(
                iter.next().context("--config requires a path")?,
            ));
        } else if !literal && arg.to_string_lossy().starts_with("--config=") {
            config = Some(PathBuf::from(
                arg.to_string_lossy().strip_prefix("--config=").unwrap(),
            ));
        } else {
            if arg == "--" {
                literal = true;
            }
            out.push(arg.clone());
        }
    }
    Ok((config, out))
}

pub fn dispatch() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let mut config = None;
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--config" {
            index += 1;
            config = args.get(index).map(PathBuf::from);
            if config.is_none() {
                return Ok(());
            }
        } else if let Some(path) = args[index]
            .to_str()
            .and_then(|s| s.strip_prefix("--config="))
        {
            config = Some(path.into());
        } else if args[index] == "--json" {
            json = true;
        } else {
            break;
        }
        index += 1;
    }
    let Some(name) = args.get(index).and_then(|s| s.to_str()) else {
        return Ok(());
    };
    if matches!(name, "--help" | "-h") || (name == "help" && index + 1 == args.len()) {
        let (config, _) = extract_config(&args, config)?;
        let path = crate::config::config_path(config.as_deref())?;
        let cfg = load_config(&path).unwrap_or_else(|err| {
            eprintln!("warning: cannot load extension configuration: {err:#}");
            Config {
                store: PathBuf::new(),
                editor: None,
                aliases: BTreeMap::new(),
            }
        });
        let catalog = discovery::catalog(&cfg, &path)?;
        if !catalog.is_empty() {
            crate::output::line("Extensions (inspect with bs aliases show NAME):")?;
            for entry in catalog.values() {
                crate::output::line(format!(
                    "  {} [{}]  {}",
                    entry.name,
                    entry.kind,
                    entry
                        .description
                        .as_deref()
                        .unwrap_or("(no description provided)")
                ))?;
            }
            crate::output::line("")?;
        }
        return Ok(());
    }
    if name.starts_with(['-', '_']) || BUILTINS.contains(&name) {
        return Ok(());
    }
    let (config, mut forwarded) = extract_config(&args[index + 1..], config)?;
    if json {
        forwarded.insert(0, "--json".into());
    }
    let path = crate::config::config_path(config.as_deref())?;
    let cfg = load_config(&path)?;
    if !cfg.aliases.contains_key(name) && !discovery::executables().contains_key(name) {
        return Ok(());
    }
    if let Some(alias) = cfg.aliases.get(name) {
        let d = alias.definition();
        if d.exec.is_none()
            && forwarded
                .iter()
                .take_while(|a| *a != "--")
                .any(|a| a == "--help" || a == "-h")
        {
            inspect(
                &cfg,
                &path,
                &crate::cli::Aliases {
                    action: Some("show".into()),
                    name: Some(name.into()),
                    args: vec![],
                },
                json,
            )?;
            std::process::exit(0);
        }
    }
    plan(name, &forwarded, &cfg, &path)?.execute(name)
}
