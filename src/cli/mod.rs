use std::path::PathBuf;
use usage::{Args, Cli, Subcommands};

/// A local Markdown-first store for reusable bits
#[derive(Cli)]
#[usage(bin = "bs", version, completion, unknown_flags = "error")]
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
    Move(Move),
    Delete(Delete),
    /// Discover extensions; inspect definitions or preview execution without running
    Aliases(Aliases),
    List(List),
    Search(Search),
    Show(Show),
    Open(Open),
    Context(Context),
    Validate(Filter),
    Prune(Prune),
    Completion(Completion),
}
/// Inspect aliases and PATH extensions without executing them
#[derive(Args)]
pub struct Aliases {
    #[usage(choices("show", "dry-run"))]
    pub action: Option<String>,
    pub name: Option<String>,
    /// Arguments to preview (after --)
    #[usage(double_dash = "required")]
    pub args: Vec<String>,
}
/// Set up a store and configuration without overwriting existing configuration
#[derive(Args)]
pub struct Init {
    #[usage(long)]
    pub store: Option<PathBuf>,
    /// Editor command with quoted arguments, e.g. 'code --wait'; no shell evaluation. Otherwise use VISUAL or EDITOR
    #[usage(long)]
    pub editor: Option<String>,
}
/// Discover, create, or delete empty shelves
#[derive(Args)]
pub struct Shelf {
    #[usage(subcommand)]
    pub command: ShelfCommands,
}
#[derive(Subcommands)]
pub enum ShelfCommands {
    List(Empty),
    Add(ShelfAdd),
    Delete(ShelfDelete),
}
/// Delete an empty shelf and its settings/guidance; refuse all other contents
#[derive(Args)]
pub struct ShelfDelete {
    #[usage(complete = complete_shelf_delete)]
    pub shelf: String,
    /// Validate and preview without changing files
    #[usage(long)]
    pub dry_run: bool,
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
    /// Positive whole days up to 36500d, e.g. 14d
    #[usage(long)]
    pub retention: Option<String>,
}
/// Save a new bit, preserving the supplied body verbatim. Requires --file, --stdin or --interactive
#[derive(Args)]
pub struct Add {
    /// Set a literal string KEY=VALUE (repeatable)
    #[usage(long)]
    pub set: Vec<String>,
    /// Set a typed JSON value KEY=JSON (repeatable)
    #[usage(long)]
    pub set_json: Vec<String>,
    /// Remove a metadata field (repeatable)
    #[usage(long)]
    pub unset: Vec<String>,

    #[usage(complete = complete_add)]
    /// Identifier: SHELF/NAME (no .md extension); required unless --interactive prompts for it
    pub id: Option<String>,
    /// Optional descriptive title; the identifier is the display name
    #[usage(long)]
    pub title: Option<String>,
    /// Comma-separated tags, e.g. rust,project:bitshelf
    #[usage(long)]
    pub tags: Option<String>,
    /// Read a body from a file, or - for stdin (use /dev/null for an empty body)
    #[usage(long)]
    pub file: Option<PathBuf>,
    /// Read the body from standard input
    #[usage(long)]
    pub stdin: bool,
    /// Prompt for missing details and write the body in your editor
    #[usage(long)]
    pub interactive: bool,
}
/// Edit a bit via your editor, or replace its body/metadata noninteractively
#[derive(Args)]
pub struct Edit {
    /// Set a literal string KEY=VALUE (repeatable)
    #[usage(long)]
    pub set: Vec<String>,
    /// Set a typed JSON value KEY=JSON (repeatable)
    #[usage(long)]
    pub set_json: Vec<String>,
    /// Remove a metadata field (repeatable); also removes an invalid created/updated
    #[usage(long)]
    pub unset: Vec<String>,

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
/// List bits, optionally sorted by creation or edit time
#[derive(Args)]
pub struct List {
    /// Include shelves excluded from default discovery
    #[usage(long)]
    pub all: bool,
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
/// Search IDs, titles, tags and bodies with ranked, case-insensitive terms
#[derive(Args)]
pub struct Search {
    /// Include shelves excluded from default discovery
    #[usage(long)]
    pub all: bool,
    /// Include optional title metadata after each ID
    #[usage(long)]
    pub long: bool,
    /// Print filesystem paths instead of IDs
    #[usage(long)]
    pub paths: bool,
    /// Terminate IDs or paths with NUL instead of newline
    #[usage(long)]
    pub null: bool,

    /// Whitespace-separated AND terms, "exact phrases", and !exclusions
    pub query: String,
    /// Match any positive term instead of all; exclusions still apply
    #[usage(long)]
    pub any: bool,
    /// Filter by an exact, case-sensitive tag
    #[usage(long)]
    pub tag: Option<String>,
    /// Sort by relevance (default) or identifier
    #[usage(long, choices("relevance", "id"))]
    pub sort: Option<String>,
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
    /// Shelf names or SHELF/NAME bit IDs (no .md extension); omit to open the store
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
/// Validate metadata without changing files; without SHELF, check all shelves,
/// including those excluded from discovery
#[derive(Args)]
pub struct Filter {
    #[usage(complete = complete_filter)]
    pub shelf: Option<String>,
}
/// Delete bits permanently without prompting; preview first with --dry-run
#[derive(Args)]
pub struct Delete {
    /// One or more exact SHELF/NAME identifiers
    #[usage(required, var_min = 1, complete = complete_delete)]
    pub id: Vec<String>,
    /// Preview without changing files
    #[usage(long)]
    pub dry_run: bool,
}

/// Apply delete/move expiry policies to explicitly expired bits on retention-enabled shelves
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
        for s in store.shelves()?.results.into_iter().filter(|s| !s.missing) {
            if shelves {
                result.push(usage::complete::Candidate::new(s.name.clone()));
            }
            if bits && (!shelves || ctx.prefix.contains('/')) {
                let Ok(entries) = std::fs::read_dir(store.bits_path(&s.name)?) else {
                    continue;
                };
                for e in entries.flatten() {
                    let path = e.path();
                    if e.file_type().is_ok_and(|t| t.is_file())
                        && store.safe(&path).is_ok()
                        && let Ok(id) = crate::identity::discovered(&s.name, &path)
                    {
                        result.push(usage::complete::Candidate::new(id));
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

/// Move a bit to a shelf or a new shelf/name ID without overwriting
#[derive(Args)]
pub struct Move {
    #[usage(complete = complete_move)]
    pub id: String,
    pub destination: String,
    #[usage(long)]
    pub title: Option<String>,
    /// Replace tags with a comma-separated list
    #[usage(long)]
    pub tags: Option<String>,
    /// Set a literal string KEY=VALUE (repeatable)
    #[usage(long)]
    pub set: Vec<String>,
    /// Set a typed JSON value KEY=JSON (repeatable)
    #[usage(long)]
    pub set_json: Vec<String>,
    /// Remove a metadata field (repeatable)
    #[usage(long)]
    pub unset: Vec<String>,
    /// Validate and preview without changing files
    #[usage(long)]
    pub dry_run: bool,
}
completer!(complete_move, Move, true, false);

completer!(complete_delete, Delete, true, false);

completer!(complete_shelf_delete, ShelfDelete, false, true);
