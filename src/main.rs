mod bit;
mod cli;
mod completion;
mod config;
mod context;
mod editor;
mod interactive;
mod output;
mod prune;
mod store;

use anyhow::{Context, Result, ensure};
use chrono::Utc;
use cli::{Bs, Commands, ShelfCommands};
use config::{Config, ShelfConfig};
use output::emit;
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs,
    io::{IsTerminal, Read},
    path::PathBuf,
};
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
    if let Err(err) = run(Bs::parse()) {
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
            store: root.clone(),
            editor: c
                .editor
                .filter(|s| !s.trim().is_empty())
                .map(|s| shell_words::split(&s))
                .transpose()?,
            shelves: BTreeMap::from([("notes".into(), ShelfConfig::default())]),
        };
        cfg.validate()?;
        fs::create_dir_all(&root)?;
        let store = Store {
            config: cfg.clone(),
        };
        fs::create_dir_all(store.shelf_path("notes", false)?)?;
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
    let mut store = Store {
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
                                " (missing; use bs shelf add)"
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
                let mut cfg = store.settings(&name);
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
                store.config.shelves.insert(name.clone(), cfg);
                store.config.validate()?;
                fs::create_dir_all(&dest)?;
                store.config.save(&path, false)?;
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
                if c.shelf.is_none() {
                    c.shelf = Some(interactive::select(
                        "Choose shelf",
                        &store
                            .shelves()?
                            .into_iter()
                            .filter(|s| !s.missing)
                            .map(|s| s.name)
                            .collect::<Vec<_>>(),
                    )?);
                }
                if c.title.is_none() {
                    c.title = Some(interactive::input("Title")?);
                }
                if c.tags.is_none() {
                    c.tags = Some(interactive::input("Tags, comma-separated")?);
                }
            }
            usage_check(
                c.shelf.is_some() && c.title.is_some(),
                "add requires SHELF and --title TEXT (or --interactive)",
            )?;
            let shelf = c.shelf.unwrap();
            let title = c.title.unwrap();
            let slug = c.slug.unwrap_or_else(|| bit::slug(&title));
            let id = format!("{shelf}/{slug}");
            let destination = store.bit_path(&id)?;
            ensure!(
                !destination.try_exists()?,
                "bit {id} already exists; choose --slug ALTERNATIVE"
            );
            let body = if let Some(file) = c.file {
                fs::read_to_string(&file)
                    .with_context(|| format!("cannot read {}", file.display()))?
            } else if c.stdin {
                let mut s = String::new();
                std::io::stdin().read_to_string(&mut s)?;
                s
            } else {
                String::new()
            };
            let cfg = store.settings(&shelf);
            let mut raw = bit::create(&title, c.tags.as_deref(), &body, &cfg, Utc::now())?;
            let mut draft = None;
            if c.interactive {
                use std::io::Write;
                let mut file = tempfile::Builder::new()
                    .prefix(".draft-")
                    .suffix(".md")
                    .tempfile_in(store.shelf_path(&shelf, true)?)?;
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
                raw = fs::read_to_string(&p)?;
                draft = Some(p);
            }
            let checked = bit::inspect(id.clone(), destination, &raw, &cfg);
            ensure!(
                checked.errors.is_empty(),
                "invalid metadata: {}{}",
                checked.errors.join(", "),
                draft
                    .as_ref()
                    .map(|p| format!("; draft preserved at {}", p.display()))
                    .unwrap_or_default()
            );
            let dest = store.write_bit(&id, &raw)?;
            if let Some(p) = draft
                && let Err(e) = fs::remove_file(&p)
            {
                eprintln!("warning: could not remove draft {}: {e}", p.display());
            }
            emit(&json!({"id":id,"path":dest}), json_output, id)
        }
        Commands::List(c) => {
            let bits: Vec<_> = store
                .bits(c.shelf.as_deref())?
                .into_iter()
                .filter(|b| c.tag.as_ref().is_none_or(|t| b.tags.contains(t)))
                .collect();
            let human = bits
                .iter()
                .map(|b| format!("{}\t{}", b.id, b.title))
                .collect::<Vec<_>>()
                .join("\n");
            emit(&bits, json_output, human)
        }
        Commands::Search(c) => {
            let query = c.query.to_lowercase();
            let bits: Vec<_> = store
                .bits(c.shelf.as_deref())?
                .into_iter()
                .filter(|b| {
                    format!("{}\n{}\n{}", b.title, b.tags.join(" "), b.body)
                        .to_lowercase()
                        .contains(&query)
                })
                .collect();
            let human = bits
                .iter()
                .map(|b| format!("{}\t{}", b.id, b.title))
                .collect::<Vec<_>>()
                .join("\n");
            emit(&bits, json_output, human)
        }
        Commands::Show(c) => {
            let path = store.bit_path(&c.id)?;
            let content =
                fs::read_to_string(&path).with_context(|| format!("cannot read bit {}", c.id))?;
            if json_output {
                emit(&json!({"id":c.id,"path":path,"content":content}), true, "")
            } else {
                print!("{content}");
                Ok(())
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
                        .bits(None)?
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
            let bits = store.bits(c.shelf.as_deref())?;
            let mut results: Vec<_> = bits.iter().map(|b| json!({"id":b.id,"path":b.path,"errors":b.errors,"valid":b.errors.is_empty()})).collect();
            if c.shelf.is_none() {
                for s in store.shelves()?.into_iter().filter(|s| s.missing) {
                    results.push(json!({"id":null,"path":s.path,"errors":[format!("missing shelf {}; use bs shelf add {}", s.name, s.name)],"valid":false}));
                }
            }
            let valid = results.iter().all(|v| v["valid"] == true);
            emit(
                &results,
                json_output,
                if valid {
                    "Validation passed"
                } else {
                    "Validation failed"
                },
            )?;
            ensure!(valid, "validation failed");
            Ok(())
        }
        Commands::Prune(c) => {
            let now = Utc::now();
            let bits = store.bits(c.shelf.as_deref())?;
            let mut results = vec![];
            let mut failed = false;
            for b in bits {
                let shelf = b.id.split('/').next().unwrap();
                match prune::decide(&b, &store.settings(shelf), now) {
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
                }
                results.push(json!({"id":b.id,"path":b.path,"status":if c.dry_run {"would_remove"} else {"removed"}}));
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
