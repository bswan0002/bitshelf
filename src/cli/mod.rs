use std::path::PathBuf;
use usage::{Args, Cli, Subcommands};

/// A local Markdown-first store for reusable bits
#[derive(Cli)]
#[usage(bin = "bs", version = "0.1.0", completion)]
pub struct Bs {
    /// Override the XDG configuration file
    #[usage(long, global)]
    pub config: Option<PathBuf>,
    /// Emit a single JSON document (no prompts)
    #[usage(long, global)]
    pub json: bool,
    #[usage(subcommand)]
    pub command: Commands,
}
#[derive(Subcommands)]
pub enum Commands {
    Init(Init),
    Shelf(Shelf),
    Add(Add),
    List(List),
    Search(Search),
    Show(Show),
    Open(Open),
    Context(Context),
    Validate(Filter),
    Prune(Prune),
    Completion(Completion),
}
/// Set up a store and configuration without overwriting existing configuration
#[derive(Args)]
pub struct Init {
    #[usage(long)]
    pub store: Option<PathBuf>,
    /// Editor override, e.g. 'code'; otherwise use VISUAL or EDITOR
    #[usage(long)]
    pub editor: Option<String>,
}
/// Discover or create shelves
#[derive(Args)]
pub struct Shelf {
    #[usage(subcommand)]
    pub command: ShelfCommands,
}
#[derive(Subcommands)]
pub enum ShelfCommands {
    List(Empty),
    Add(ShelfAdd),
}
#[derive(Args)]
pub struct Empty {}
/// Create a shelf; existing contents are preserved
#[derive(Args)]
pub struct ShelfAdd {
    pub name: Option<String>,
    #[usage(long)]
    pub description: Option<String>,
    /// Comma-separated built-in metadata fields
    #[usage(long)]
    pub required: Option<String>,
    /// Positive whole days, e.g. 14d
    #[usage(long)]
    pub retention: Option<String>,
}
/// Save a new bit, preserving the supplied body verbatim
#[derive(Args)]
pub struct Add {
    #[usage(complete = complete_add)]
    pub shelf: Option<String>,
    #[usage(long)]
    pub title: Option<String>,
    #[usage(long)]
    pub tags: Option<String>,
    #[usage(long)]
    pub slug: Option<String>,
    #[usage(long)]
    pub file: Option<PathBuf>,
    #[usage(long)]
    pub stdin: bool,
    #[usage(long)]
    pub interactive: bool,
}
/// List bits in deterministic identifier order
#[derive(Args)]
pub struct List {
    #[usage(complete = complete_list)]
    pub shelf: Option<String>,
    #[usage(long)]
    pub tag: Option<String>,
}
/// Search titles, tags and bodies (case-insensitive plain text)
#[derive(Args)]
pub struct Search {
    pub query: String,
    #[usage(long, complete = complete_search)]
    pub shelf: Option<String>,
}
/// Print complete, unmodified Markdown
#[derive(Args)]
pub struct Show {
    #[usage(complete = complete_show)]
    pub id: String,
}
/// Open the store, shelves or bits in your editor
#[derive(Args)]
pub struct Open {
    #[usage(complete = complete_open)]
    pub target: Vec<String>,
    #[usage(long)]
    pub pick: bool,
}
/// Read shelf requirements and full SHELF.md guidance
#[derive(Args)]
pub struct Context {
    #[usage(complete = complete_context)]
    pub shelf: String,
}
/// Validate metadata without changing files
#[derive(Args)]
pub struct Filter {
    #[usage(complete = complete_filter)]
    pub shelf: Option<String>,
}
/// Remove explicitly expired bits only from retention-enabled shelves
#[derive(Args)]
pub struct Prune {
    #[usage(complete = complete_prune)]
    pub shelf: Option<String>,
    #[usage(long)]
    pub dry_run: bool,
}
/// Print a self-contained runtime shell completion script
#[derive(Args)]
pub struct Completion {
    #[usage(long, choices("bash", "zsh", "fish", "elvish", "nu", "powershell"))]
    pub shell: String,
}
fn candidates(
    bits: bool,
    shelves: bool,
    ctx: &usage::complete::CompleteCtx<'_>,
) -> Vec<usage::complete::Candidate<'static>> {
    // Usage supplies decoded shell words, including inherited global flags.
    let args = &ctx.words[..ctx.cword];
    let override_path = args.iter().enumerate().rev().find_map(|(i, a)| {
        a.strip_prefix("--config=").map(PathBuf::from).or_else(|| {
            if a == "--config" {
                args.get(i + 1).map(PathBuf::from)
            } else {
                None
            }
        })
    });
    let load = || -> anyhow::Result<Vec<usage::complete::Candidate<'static>>> {
        let config =
            crate::config::Config::load(&crate::config::config_path(override_path.as_deref())?)?;
        let store = crate::store::Store { config };
        let mut result = vec![];
        for s in store.shelves()?.into_iter().filter(|s| !s.missing) {
            if shelves {
                result.push(usage::complete::Candidate::new(s.name.clone()));
            }
            if bits && (!shelves || ctx.prefix.contains('/')) {
                for e in std::fs::read_dir(&s.path)? {
                    let e = e?;
                    let p = e.path();
                    if e.file_type()?.is_file()
                        && p.extension().is_some_and(|e| e == "md")
                        && e.file_name() != "SHELF.md"
                        && !e.file_name().to_string_lossy().starts_with('.')
                    {
                        result.push(usage::complete::Candidate::new(format!(
                            "{}/{}",
                            s.name,
                            p.file_stem().unwrap().to_string_lossy()
                        )));
                    }
                }
            }
        }
        Ok(result)
    };
    load().unwrap_or_default()
}
macro_rules! completer {
    ($fn:ident, $ty:ty, $bits:expr, $shelves:expr) => {
        fn $fn(
            _: &<$ty as usage::spec::CommandArgs>::Partial,
            ctx: &usage::complete::CompleteCtx<'_>,
        ) -> Vec<usage::complete::Candidate<'static>> {
            candidates($bits, $shelves, ctx)
        }
    };
}
completer!(complete_add, Add, false, true);
completer!(complete_list, List, false, true);
completer!(complete_search, Search, false, true);
completer!(complete_show, Show, true, false);
completer!(complete_open, Open, true, true);
completer!(complete_context, Context, false, true);
completer!(complete_filter, Filter, false, true);
completer!(complete_prune, Prune, false, true);
