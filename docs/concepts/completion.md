# Shell completion

Completion suggests shelves and bit IDs as you type: `bs open ui/<Tab>` completes bits, `bs add <Tab>` offers shelf prefixes such as `ui/` (then type a new name), and `--shelf <Tab>` completes filters.

## Install

```sh
bs completion install                   # detect from $SHELL; preview and confirm
bs completion install --shell zsh       # override detection
bs completion install --dry-run         # preview without writing or prompting
bs completion install --yes             # approve without prompting (scripts/CI)
bs completion uninstall                 # preview and confirm removal of managed setup
```

Then **start a new shell**, since an executable can't update its parent shell. `bs context <Tab>` should suggest shelves, and fzf-based completion UIs can display the suggestions.

Automatic setup supports Bash, Zsh, and Fish, and targets one shell. If `$SHELL` differs from the shell you're running, pass `--shell`.

- **Bash:** appends a managed block to `~/.bashrc`. Login shells must already source `.bashrc` from their profile; the installer doesn't edit profiles.
- **Zsh:** appends a managed block to `$ZDOTDIR/.zshrc` (otherwise `~/.zshrc`). The block runs `compinit` only if `compdef` isn't already available, then loads the completion script.
- **Fish:** writes a managed `bs.fish` under `$XDG_CONFIG_HOME/fish/completions` (otherwise `~/.config/fish/completions`). Rerun installation after upgrading to refresh it.

## Behavior and limits

- Repeating installation doesn't duplicate setup. Existing exact Bash/Zsh activation lines from the manual instructions below are recognized and left alone.
- Uninstall removes only managed blocks and files, not manual or package-manager setup.
- Symlinked dotfiles and configuration directories are supported: the preview shows the resolved target, changes update that target, and links remain intact. Uninstall leaves a symlinked Fish target empty rather than breaking its link.
- Broken or cyclic links, non-file targets, existing nonempty unmanaged Fish files, and malformed markers are rejected rather than overwritten.
- Unrelated rc content and permissions are preserved; a missing final newline is added when appending.
- Installation doesn't set up a store or install the agent skill. `--json` isn't supported for completion commands.
- Completions call the installed `bs` at Tab time, so new shelves and files appear immediately. Config overrides typed before the cursor are honored, and completion never changes a store.

## Manual setup

```sh
# bash startup
source <(bs completion --shell bash)
# zsh startup (after `autoload -Uz compinit; compinit`)
eval "$(bs completion --shell zsh)"
# fish
mkdir -p ~/.config/fish/completions
bs completion --shell fish > ~/.config/fish/completions/bs.fish
```

Elvish, Nushell, and PowerShell scripts are also generated. Activating them is left to those shells' own setup.
