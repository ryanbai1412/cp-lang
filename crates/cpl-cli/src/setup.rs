//! `cpl setup <editor>`: writes the editor files built into this binary
//! where the editor loads them, so one command sets an editor up.

use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) const USAGE: &str = "\
usage: cpl setup <neovim | vim | vscode> [--dir DIR | --vsix FILE]

Writes CP Lang's editor files where the editor loads them at startup, then
says where. Neovim gets the filetype, highlighting and `cpl lsp`:
diagnostics, completion, `gd` to the definition and formatting on save.
Vim gets the filetype and highlighting, and the server through vim-lsp or
yegappan/lsp when one is installed. VS Code gets the extension from the
edge release, installed with `code --install-extension` (`$CPL_CODE` names
another editor command, such as `codium` or `cursor`). Run it again after
`cpl self update`.

options:
  --dir DIR     write into DIR instead (a `pack/*/start/<name>` folder)
  --vsix FILE   install this extension file instead of downloading it
";

const LUA: (&str, &str) = (
    "plugin/cplang.lua",
    include_str!("../../../editors/neovim/plugin/cplang.lua"),
);
const COMMON: &[(&str, &str)] = &[
    (
        "ftdetect/cp.vim",
        include_str!("../../../editors/neovim/ftdetect/cp.vim"),
    ),
    (
        "ftplugin/cp.vim",
        include_str!("../../../editors/neovim/ftplugin/cp.vim"),
    ),
    (
        "syntax/cp.vim",
        include_str!("../../../editors/neovim/syntax/cp.vim"),
    ),
    (
        "plugin/cplang.vim",
        include_str!("../../../editors/neovim/plugin/cplang.vim"),
    ),
];
const QUERIES: (&str, &str) = (
    "queries/cp/highlights.scm",
    include_str!("../../../editors/tree-sitter-cp/queries/highlights.scm"),
);

/// The files for `editor`, by path under its package folder.
fn files(editor: &str) -> Option<Vec<(&'static str, &'static str)>> {
    let mut files = COMMON.to_vec();
    match editor {
        "neovim" | "nvim" => files.extend([LUA, QUERIES]),
        "vim" => {}
        _ => return None,
    }
    Some(files)
}

/// Where `editor` loads a start package from: Neovim's
/// `stdpath('data')/site/pack`, Vim's `~/.vim/pack` (`~/vimfiles` on Windows).
fn package_dir(editor: &str) -> Option<PathBuf> {
    let var = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let home = var("HOME").or_else(|| var("USERPROFILE"));
    let site = match editor {
        "vim" if cfg!(windows) => home?.join("vimfiles"),
        "vim" => home?.join(".vim"),
        _ if cfg!(windows) => var("LOCALAPPDATA")?.join("nvim-data").join("site"),
        _ => var("XDG_DATA_HOME")
            .or_else(|| Some(home?.join(".local").join("share")))?
            .join("nvim")
            .join("site"),
    };
    Some(site.join("pack").join("cpl").join("start").join("cplang"))
}

pub(crate) fn setup(args: &[&str], stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let mut editor = None;
    let mut dir = None;
    let mut vsix = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match *arg {
            "--dir" => match rest.next() {
                Some(d) => dir = Some(PathBuf::from(d)),
                None => return usage(stderr, "`--dir` needs a folder"),
            },
            "--vsix" => match rest.next() {
                Some(f) => vsix = Some(PathBuf::from(f)),
                None => return usage(stderr, "`--vsix` needs a file"),
            },
            e if editor.is_none() && !e.starts_with('-') => editor = Some(e),
            other => return usage(stderr, &format!("unexpected `{other}`")),
        }
    }
    let Some(editor) = editor else {
        return usage(
            stderr,
            "name the editor: `cpl setup neovim` or `cpl setup vim`",
        );
    };
    if editor == "vscode" || editor == "code" {
        return match install_vscode(vsix) {
            Ok(done) => {
                let _ = writeln!(stdout, "{done}");
                0
            }
            Err(e) => {
                *stderr += &format!("cpl: {e}\n");
                1
            }
        };
    }
    let Some(files) = files(editor) else {
        return usage(
            stderr,
            &format!("no setup for `{editor}`: `neovim`, `vim` or `vscode`"),
        );
    };
    let Some(dir) = dir.or_else(|| package_dir(editor)) else {
        *stderr += "cpl: no home folder to install into; pass --dir\n";
        return 1;
    };
    if let Err(e) = write_all(&dir, &files) {
        *stderr += &format!("cpl: can't write {}: {e}\n", dir.display());
        return 1;
    }
    let _ = writeln!(
        stdout,
        "installed CP Lang for {editor} in {}\nopen a .cp file to start; `cpl` must be on the PATH (or set g:cpl_path)",
        dir.display()
    );
    0
}

/// Installs the extension in `vsix`, or the edge release's, into VS Code.
fn install_vscode(vsix: Option<PathBuf>) -> Result<String, String> {
    let code = std::env::var("CPL_CODE").ok().filter(|c| !c.is_empty());
    let code = code.unwrap_or_else(|| if cfg!(windows) { "code.cmd" } else { "code" }.into());
    let downloaded = std::env::temp_dir().join(format!("cp-lang-{}.vsix", std::process::id()));
    let file = match vsix {
        Some(file) => file,
        None => {
            let url = format!(
                "{}/download/edge/cp-lang.vsix",
                crate::toolchain::releases_url()
            );
            crate::toolchain::fetch(&url, &downloaded)?;
            downloaded.clone()
        }
    };
    let out = std::process::Command::new(&code)
        .arg("--install-extension")
        .arg(&file)
        .arg("--force")
        .output();
    let _ = std::fs::remove_file(&downloaded);
    let out = out.map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => format!(
            "no `{code}` on the PATH: in VS Code run \"Shell Command: Install 'code' command in PATH\", or set CPL_CODE"
        ),
        _ => format!("could not run `{code}`: {e}"),
    })?;
    if !out.status.success() {
        return Err(format!(
            "`{code} --install-extension` failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(format!(
        "installed CP Lang for VS Code from {}\nopen a .cp file to start; `cpl` must be on the PATH (or set `cp.server.path`)",
        file.display()
    ))
}

fn write_all(dir: &Path, files: &[(&str, &str)]) -> std::io::Result<()> {
    for (path, text) in files {
        let path = dir.join(path);
        std::fs::create_dir_all(path.parent().unwrap_or(dir))?;
        std::fs::write(path, text)?;
    }
    Ok(())
}

fn usage(stderr: &mut String, message: &str) -> u8 {
    *stderr += &format!("cpl: {message}\n\n{USAGE}");
    crate::EXIT_USAGE
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every runtime file in editors/neovim ships, so a new one can't be left out.
    #[test]
    fn every_editor_file_ships() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../editors/neovim");
        let shipped: Vec<&str> = files("neovim").unwrap().iter().map(|(p, _)| *p).collect();
        for sub in ["ftdetect", "ftplugin", "syntax", "plugin", "queries/cp"] {
            for entry in std::fs::read_dir(root.join(sub)).unwrap() {
                let name = format!("{sub}/{}", entry.unwrap().file_name().to_string_lossy());
                assert!(
                    shipped.contains(&name.as_str()),
                    "{name} is not in `cpl setup neovim`"
                );
            }
        }
        assert!(!files("vim").unwrap().contains(&LUA));
    }
}
