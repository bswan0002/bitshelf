mod aliases;
mod bit;
mod cli;
mod completion;
mod config;
mod context;
mod editor;
mod input;
mod interactive;
mod lifecycle;
mod moving;
mod output;
mod prune;
mod store;

use anyhow::{Context, Result, ensure};
use chrono::Utc;
use cli::{Bs, Commands, ShelfCommands};
use config::{Config, ShelfConfig};
use output::emit;
use serde_json::json;
use std::{fs, io::IsTerminal, path::PathBuf};
use store::Store;

#[derive(Debug)]
struct UsageError(String);
impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for UsageError {}
fn usage_check(condition: bool, message: &str) -> Result<()> {
    if !condition {
        return Err(UsageError(message.into()).into());
    }
    Ok(())
}
fn main() {
    if let Err(err) = aliases::dispatch().and_then(|()| run(Bs::parse())) {
        if err
            .downcast_ref::<std::io::Error>()
            .is_some_and(|e| e.kind() == std::io::ErrorKind::BrokenPipe)
        {
            return;
        }
        eprintln!("error: {err:#}");
        std::process::exit(if err.downcast_ref::<UsageError>().is_some() {
            2
        } else {
            1
        });
    }
}
fn run(args: Bs) -> Result<()> {
    let json_output = args.json;
    if let Commands::Completion(c) = &args.command {
        usage_check(!json_output, "completion does not support --json")?;
        return completion::run(c);
    }
    let path = config::config_path(args.config.as_deref())?;
    if let Commands::Init(mut c) = args.command {
        ensure!(
            !path.try_exists()?,
            "configuration already exists: {}",
            path.display()
        );
        if c.store.is_none() && !json_output && std::io::stdin().is_terminal() {
            interactive::require(json_output)?;
            let v = interactive::input("Store directory (empty for ~/bitshelf)")?;
            c.store = Some(PathBuf::from(if v.is_empty() { "~/bitshelf" } else { &v }));
            if c.editor.is_none() && editor::environment_editor().is_none() {
                c.editor = Some(interactive::input(
                    "Editor command (e.g. code or vim; empty to configure later)",
                )?);
            }
        }
        usage_check(
            c.store.is_some(),
            "bs init requires --store PATH outside an interactive terminal",
        )?;
        let root = config::resolve(&c.store.unwrap(), path.parent().unwrap())?;
        let cfg = Config {
            aliases: Default::default(),
            store: root.clone(),
            editor: c
                .editor
                .filter(|s| !s.trim().is_empty())
                .map(|s| shell_words::split(&s))
                .transpose()?,
        };
        cfg.validate()?;
        fs::create_dir_all(&root)?;
        let store = Store {
            config: cfg.clone(),
        };
        let shelf = store.shelf_path("notes", false)?;
        store.safe(&shelf.join("bits"))?;
        store.safe(&shelf.join("bs.toml"))?;
        fs::create_dir_all(shelf.join("bits"))?;
        if !shelf.join("bs.toml").try_exists()? {
            ShelfConfig::default().save(&shelf.join("bs.toml"))?;
        }
        cfg.save(&path, true)?;
        return emit(
            &json!({"config":path,"store":root}),
            json_output,
            format!(
                "Initialized {}\nConfiguration: {}",
                root.display(),
                path.display()
            ),
        );
    }
    let store = Store {
        config: Config::load(&path)?,
    };
    match args.command {
        Commands::Shelf(s) => match s.command {
            ShelfCommands::List(_) => {
                let shelves = store.shelves()?;
                let human = shelves
                    .iter()
                    .map(|s| {
                        format!(
                            "{}{}\t{}\t{}",
                            s.name,
                            if s.missing {
                                " (missing bits directory; use bs shelf add)"
                            } else {
                                ""
                            },
                            s.path.display(),
                            s.description.as_deref().unwrap_or("")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                emit(&shelves, json_output, human)
            }
            ShelfCommands::Add(mut c) => {
                if c.name.is_none() {
                    usage_check(
                        !json_output && std::io::stdin().is_terminal(),
                        "shelf add requires NAME outside an interactive terminal",
                    )?;
                    interactive::require(json_output)?;
                    c.name = Some(interactive::input("Shelf name")?);
                    c.description = Some(interactive::input("Description (optional)")?);
                    c.required = Some(interactive::input(
                        "Required fields, comma-separated (optional)",
                    )?);
                    c.retention = Some(interactive::input(
                        "Retention, e.g. 14d (empty for permanent)",
                    )?)
                    .filter(|s| !s.is_empty());
                }
                let name = c.name.unwrap();
                let dest = store.shelf_path(&name, false)?;
                let mut cfg = store.settings(&name)?;
                if let Some(v) = c.description {
                    cfg.description = Some(v);
                }
                if let Some(v) = c.required {
                    cfg.required = v
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_owned)
                        .collect();
                }
                if c.retention.is_some() {
                    cfg.retention = c.retention;
                }
                cfg.validate()?;
                store.safe(&dest.join("bits"))?;
                fs::create_dir_all(dest.join("bits"))?;
                cfg.save(&dest.join("bs.toml"))?;
                emit(
                    &json!({"name":name,"path":dest}),
                    json_output,
                    dest.display().to_string(),
                )
            }
        },
        Commands::Add(mut c) => {
            usage_check(
                !(c.file.is_some() && c.stdin),
                "--file and --stdin are mutually exclusive",
            )?;
            usage_check(
                !(c.interactive && (c.file.is_some() || c.stdin)),
                "--interactive cannot be combined with --file or --stdin",
            )?;
            if c.interactive {
                interactive::require(json_output)?;
                if c.id.is_none() {
                    let shelf = interactive::select(
                        "Choose shelf",
                        &store
                            .shelves()?
                            .into_iter()
                            .filter(|s| !s.missing)
                            .map(|s| s.name)
                            .collect::<Vec<_>>(),
                    )?;
                    let name = interactive::input("Bit name (no .md extension)")?;
                    c.id = Some(format!("{shelf}/{name}"));
                }
                if c.tags.is_none() {
                    c.tags = Some(interactive::input("Tags, comma-separated")?);
                }
            }
            usage_check(
                c.id.is_some(),
                "add requires ID (shelf/bit-name) or --interactive",
            )?;
            let id = c.id.unwrap();
            let destination = store.bit_path(&id)?;
            let shelf = id.split_once('/').unwrap().0;
            ensure!(
                !destination.try_exists()?,
                "bit {id} already exists; choose a different ID"
            );
            let body = input::body(c.file.as_deref(), c.stdin)?.unwrap_or_default();
            let cfg = store.settings(shelf)?;
            let created_at = Utc::now();
            let mut raw = bit::create(
                c.title.as_deref(),
                c.tags.as_deref(),
                &body,
                &cfg,
                created_at,
            )?;
            let mut draft = None;
            if c.interactive {
                use std::io::Write;
                let mut file = tempfile::Builder::new()
                    .prefix(".draft-")
                    .suffix(".md")
                    .tempfile_in(store.bits_path(shelf)?)?;
                file.write_all(raw.as_bytes())?;
                let (_, p) = file.keep()?;
                // Keep the draft on every editor/validation/finalization failure.
                eprintln!("Draft: {}", p.display());
                editor::launch(
                    &store.config,
                    std::slice::from_ref(&p),
                    false,
                    editor::Purpose::Draft,
                )
                .with_context(|| format!("draft preserved at {}", p.display()))?;
                store
                    .safe(&p)
                    .with_context(|| format!("unsafe draft; recovery path: {}", p.display()))?;
                raw = fs::read_to_string(&p).with_context(|| {
                    format!("cannot read draft; recovery path: {}", p.display())
                })?;
                draft = Some(p);
            }
            let saved = (|| -> Result<PathBuf> {
                raw = lifecycle::new_bit(&raw, created_at)?;
                let checked = bit::inspect(id.clone(), destination, &raw, &cfg);
                ensure!(
                    checked.errors.is_empty(),
                    "invalid metadata: {}",
                    checked.errors.join(", "),
                );
                let mut state = lifecycle::State::load(&store, true)?;
                state.remember(&id, &raw)?;
                let dest = store.write_bit(&id, &raw)?;
                state.save_after_bit(&store, &id)?;
                Ok(dest)
            })();
            let dest = saved.with_context(|| match &draft {
                Some(p) => format!("add did not complete; draft preserved at {}", p.display()),
                None => "add did not complete".into(),
            })?;
            if let Some(p) = draft {
                lifecycle::cleanup_draft(&p);
            }
            emit(&json!({"id":id,"path":dest}), json_output, id)
        }
        Commands::Move(c) => {
            let result = moving::run(&store, c)?;
            let id = result["id"].as_str().unwrap_or("");
            let human = if result["dry_run"] == true {
                format!(
                    "Would move {} -> {id} (no files changed)",
                    result["from"].as_str().unwrap_or("")
                )
            } else {
                id.to_owned()
            };
            emit(&result, json_output, human)
        }
        Commands::Aliases(_) => emit(
            &store.config.aliases,
            json_output,
            toml::to_string_pretty(&store.config.aliases)?,
        ),
        Commands::Edit(c) => {
            let result = lifecycle::edit(&store, c, json_output)?;
            emit(&result, json_output, result["id"].as_str().unwrap_or(""))
        }
        Commands::Sync(c) => {
            let results = lifecycle::sync(&store, c.shelf.as_deref(), c.dry_run)?;
            let failed = results.iter().any(|r| r.error.is_some());
            let human = results
                .iter()
                .map(|r| {
                    format!(
                        "{}\t{}",
                        r.id,
                        r.error.as_deref().unwrap_or(if r.baselined {
                            "baselined"
                        } else if r.metadata_changed {
                            "timestamps reconciled"
                        } else {
                            "unchanged"
                        })
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let human = if c.dry_run {
                format!("Dry run (no files changed)\n{human}")
            } else {
                human
            };
            emit(
                &json!({"dry_run":c.dry_run,"results":results}),
                json_output,
                human,
            )?;
            ensure!(!failed, "some bits could not be synced; see per-bit errors");
            Ok(())
        }
        Commands::List(c) => {
            output::check_listing(json_output, c.long, c.paths, c.null)?;
            let mut bits: Vec<_> = store
                .discover(c.shelf.as_deref(), c.all)?
                .into_iter()
                .filter(|b| c.tag.as_ref().is_none_or(|t| b.tags.contains(t)))
                .collect();
            if let Some(field @ ("created" | "updated")) = c.sort.as_deref() {
                let date = |b: &bit::Bit| {
                    b.metadata
                        .get(field)
                        .and_then(|v| v.as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                };
                bits.sort_by(|a, b| match (date(a), date(b)) {
                    (Some(a_time), Some(b_time)) => {
                        let order = a_time.cmp(&b_time).then_with(|| a.id.cmp(&b.id));
                        if c.reverse { order.reverse() } else { order }
                    }
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.id.cmp(&b.id),
                });
            } else if c.reverse {
                bits.reverse();
            }
            output::listing(&bits, json_output, c.long, c.paths, c.null)
        }
        Commands::Search(c) => {
            output::check_listing(json_output, c.long, c.paths, c.null)?;
            let query = c.query.to_lowercase();
            let bits: Vec<_> = store
                .discover(c.shelf.as_deref(), c.all)?
                .into_iter()
                .filter(|b| {
                    format!(
                        "{}\n{}\n{}\n{}",
                        b.id,
                        b.title.as_deref().unwrap_or(""),
                        b.tags.join(" "),
                        b.body
                    )
                    .to_lowercase()
                    .contains(&query)
                })
                .collect();
            output::listing(&bits, json_output, c.long, c.paths, c.null)
        }
        Commands::Show(c) => {
            let path = store.bit_path(&c.id)?;
            let content =
                fs::read_to_string(&path).with_context(|| format!("cannot read bit {}", c.id))?;
            let content = if c.body {
                bit::parse(&content)?.1
            } else {
                &content
            };
            if json_output {
                emit(&json!({"id":c.id,"path":path,"content":content}), true, "")
            } else {
                output::write(content.as_bytes())
            }
        }
        Commands::Context(c) => {
            let value = context::load(&store, &c.shelf)?;
            emit(&value, json_output, serde_json::to_string_pretty(&value)?)
        }
        Commands::Open(mut c) => {
            usage_check(
                !(c.pick && !c.target.is_empty()),
                "--pick cannot be combined with explicit targets",
            )?;
            if c.pick {
                interactive::require(json_output)?;
                c.target = interactive::pick(
                    &store
                        .discover(None, false)?
                        .into_iter()
                        .map(|b| b.id)
                        .collect::<Vec<_>>(),
                )?;
            }
            let targets: Vec<_> = if c.target.is_empty() {
                ensure!(store.config.store.is_dir(), "store does not exist");
                vec![store.config.store.clone()]
            } else {
                c.target
                    .iter()
                    .map(|s| {
                        if s.contains('/') {
                            let p = store.bit_path(s)?;
                            ensure!(p.is_file(), "missing bit: {s}");
                            Ok(p)
                        } else {
                            store.shelf_path(s, true)
                        }
                    })
                    .collect::<Result<_>>()?
            };
            editor::launch(&store.config, &targets, json_output, editor::Purpose::Open)?;
            emit(&json!({"paths":targets,"opened":true}), json_output, "")
        }
        Commands::Validate(c) => {
            let results = store.validate(c.shelf.as_deref())?;
            let valid = results.iter().all(|v| v.valid);
            let human = if valid {
                "Validation passed".into()
            } else {
                results
                    .iter()
                    .filter(|r| !r.valid)
                    .map(|r| format!("{}: {}", r.path.display(), r.errors.join(", ")))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            emit(&results, json_output, human)?;
            ensure!(valid, "validation failed");
            Ok(())
        }
        Commands::Prune(c) => {
            let mut state = if c.dry_run {
                None
            } else {
                Some(lifecycle::State::load(&store, true)?)
            };
            let now = Utc::now();
            let bits = store.bits(c.shelf.as_deref())?;
            let mut results = vec![];
            let mut failed = false;
            for b in bits {
                let shelf = b.id.split('/').next().unwrap();
                match prune::decide(&b, &store.settings(shelf)?, now) {
                    prune::Decision::Keep => continue,
                    prune::Decision::Skip(error) => {
                        eprintln!("warning: {}: {error}; skipped", b.id);
                        results.push(
                            json!({"id":b.id,"path":b.path,"status":"skipped","error":error}),
                        );
                        failed = true;
                        continue;
                    }
                    prune::Decision::Remove => (),
                }
                store.safe(&b.path)?;
                if !c.dry_run {
                    fs::remove_file(&b.path)
                        .with_context(|| format!("cannot remove {}", b.path.display()))?;
                    state.as_mut().unwrap().forget(&b.id);
                }
                results.push(json!({"id":b.id,"path":b.path,"status":if c.dry_run {"would_remove"} else {"removed"}}));
            }
            if let Some(state) = state {
                state.save(&store).context(
                    "pruning may have removed files, but tracking state could not be saved; fix the error and run bs sync",
                )?;
            }
            let human = results
                .iter()
                .map(|r| {
                    format!(
                        "{}\t{}",
                        r["status"].as_str().unwrap(),
                        r["id"].as_str().unwrap()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            emit(&results, json_output, human)?;
            ensure!(
                !failed,
                "some bits were skipped; fix expiration metadata before retrying"
            );
            Ok(())
        }
        Commands::Init(_) | Commands::Completion(_) => unreachable!(),
    }
}
