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
    Edit(Edit),
    Sync(Sync),
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
    /// Identifier: shelf/bit-name (no .md extension)
    pub id: Option<String>,
    /// Optional descriptive title; the identifier is the display name
    #[usage(long)]
    pub title: Option<String>,
    #[usage(long)]
    pub tags: Option<String>,
    /// Read a body from a file, or - for stdin
    #[usage(long)]
    pub file: Option<PathBuf>,
    #[usage(long)]
    pub stdin: bool,
    #[usage(long)]
    pub interactive: bool,
}
/// Edit a bit via your editor, or replace its body/metadata noninteractively
#[derive(Args)]
pub struct Edit {
    #[usage(complete = complete_edit)]
    pub id: String,
    /// Read a body from a file, or - for stdin
    #[usage(long)]
    pub file: Option<PathBuf>,
    #[usage(long)]
    pub stdin: bool,
    #[usage(long)]
    pub title: Option<String>,
    /// Replace tags with a comma-separated list
    #[usage(long)]
    pub tags: Option<String>,
}
/// Reconcile timestamps after direct filesystem edits; first run establishes a baseline
#[derive(Args)]
pub struct Sync {
    #[usage(complete = complete_sync)]
    pub shelf: Option<String>,
    #[usage(long)]
    pub dry_run: bool,
}
/// List bits, optionally sorted by creation or edit time
#[derive(Args)]
pub struct List {
    /// Include optional title metadata after each ID
    #[usage(long)]
    pub long: bool,
    /// Print filesystem paths instead of IDs
    #[usage(long)]
    pub paths: bool,
    /// Terminate IDs or paths with NUL instead of newline
    #[usage(long)]
    pub null: bool,

    /// Sort ascending by identifier (default), creation time or last edit
    #[usage(long, choices("id", "created", "updated"))]
    pub sort: Option<String>,
    /// Reverse the selected order (newest first for timestamps)
    #[usage(long)]
    pub reverse: bool,
    #[usage(complete = complete_list)]
    pub shelf: Option<String>,
    #[usage(long)]
    pub tag: Option<String>,
}
/// Search IDs, titles, tags and bodies (case-insensitive plain text)
#[derive(Args)]
pub struct Search {
    /// Include optional title metadata after each ID
    #[usage(long)]
    pub long: bool,
    /// Print filesystem paths instead of IDs
    #[usage(long)]
    pub paths: bool,
    /// Terminate IDs or paths with NUL instead of newline
    #[usage(long)]
    pub null: bool,

    pub query: String,
    #[usage(long, complete = complete_search)]
    pub shelf: Option<String>,
}
/// Print complete, unmodified Markdown
#[derive(Args)]
pub struct Show {
    /// Print only the body, without frontmatter
    #[usage(long)]
    pub body: bool,
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
/// Generate completions or install/uninstall shell setup
#[derive(Args)]
pub struct Completion {
    /// Omit to print a script; install/uninstall support bash, zsh and fish
    #[usage(choices("install", "uninstall"))]
    pub action: Option<String>,
    /// Shell to configure; install/uninstall default to SHELL
    #[usage(long, choices("bash", "zsh", "fish", "elvish", "nu", "powershell"))]
    pub shell: Option<String>,
    /// Preview install/uninstall without changing files
    #[usage(long)]
    pub dry_run: bool,
    /// Approve install/uninstall without prompting
    #[usage(long)]
    pub yes: bool,
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
                for e in std::fs::read_dir(store.bits_path(&s.name)?)? {
                    let e = e?;
                    let p = e.path();
                    if e.file_type()?.is_file()
                        && p.extension().is_some_and(|e| e == "md")
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
fn complete_add(
    _: &<Add as usage::spec::CommandArgs>::Partial,
    ctx: &usage::complete::CompleteCtx<'_>,
) -> Vec<usage::complete::Candidate<'static>> {
    candidates(false, true, ctx)
        .into_iter()
        .map(|c| usage::complete::Candidate::new(format!("{}/", c.value)))
        .collect()
}
completer!(complete_list, List, false, true);
completer!(complete_search, Search, false, true);
completer!(complete_show, Show, true, false);
completer!(complete_open, Open, true, true);
completer!(complete_context, Context, false, true);
completer!(complete_filter, Filter, false, true);
completer!(complete_prune, Prune, false, true);

completer!(complete_edit, Edit, true, false);
completer!(complete_sync, Sync, false, true);
