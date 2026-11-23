//! `cpl setup <editor>`: writes the editor files built into this binary
//! where the editor loads them, so one command sets an editor up.

use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) const USAGE: &str = "\
usage: cpl setup <neovim | vim | vscode | jetbrains> [OPTIONS]

Writes CP Lang's editor files where the editor loads them at startup, then
says where. Neovim gets the filetype, highlighting and `cpl lsp`:
diagnostics, completion, `gd` to the definition and formatting on save.
Vim gets the filetype and highlighting, and the server through vim-lsp or
yegappan/lsp when one is installed. VS Code gets the extension from the
edge release, installed with `code --install-extension` (`$CPL_CODE` names
another editor command, such as `codium` or `cursor`). Run it again after
`cpl self update`. JetBrains IDEs get the edge release's plugin unpacked
into every installed IDE's plugins folder; restart the IDE. Its LSP features
need a commercial IDE (IDEA Ultimate, CLion, PyCharm Professional, ...).

options:
  --dir DIR     write into DIR instead (a `pack/*/start/<name>` folder)
  --vsix FILE   install this extension file instead of downloading it
  --zip FILE    install this JetBrains plugin file instead of downloading it
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
        "compiler/cpl.vim",
        include_str!("../../../editors/neovim/compiler/cpl.vim"),
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
    let home = cpl_check::home_dir();
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
    let mut zip = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match *arg {
            "--dir" => match rest.next() {
                Some(d) => dir = Some(PathBuf::from(d)),
                None => return usage(stderr, "`--dir` needs a folder"),
            },
            "--zip" => match rest.next() {
                Some(f) => zip = Some(PathBuf::from(f)),
                None => return usage(stderr, "`--zip` needs a file"),
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
    let installed = match editor {
        "vscode" | "code" => Some(install_vscode(vsix)),
        "jetbrains" | "intellij" | "idea" => Some(install_jetbrains(zip, dir.take())),
        _ => None,
    };
    if let Some(installed) = installed {
        return match installed {
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
            &format!("no setup for `{editor}`: `neovim`, `vim`, `vscode` or `jetbrains`"),
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
    if matches!(editor, "neovim" | "nvim") {
        let version = std::process::Command::new("nvim").arg("--version").output();
        if let Some(warning) = version
            .ok()
            .and_then(|out| old_neovim(&String::from_utf8_lossy(&out.stdout)))
        {
            *stderr += &warning;
        }
    }
    0
}

/// A warning when `nvim --version` printed a Neovim older than the 0.10 the
/// plugin needs (Ubuntu 22.04's apt has 0.6), with where to get a new one.
fn old_neovim(version: &str) -> Option<String> {
    let number = version.lines().next()?.strip_prefix("NVIM v")?;
    let mut parts = number
        .split(|c: char| !c.is_ascii_digit())
        .map(|p| p.parse::<u32>().ok());
    let (major, minor) = (parts.next()??, parts.next()??);
    ((major, minor) < (0, 10)).then(|| {
        format!(
            "cpl: warning: this is Neovim {major}.{minor}, but CP Lang needs 0.10 or later; get it from \
             https://github.com/neovim/neovim/releases (Linux: the .appimage or tarball; macOS: \
             `brew install neovim`; Windows: `winget install Neovim.Neovim`)\n"
        )
    })
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

/// Each installed JetBrains IDE's plugins folder, from its config folders
/// (`IntelliJIdea2024.3`, `CLion2025.1`, ...).
fn jetbrains_plugin_dirs() -> Vec<PathBuf> {
    let var = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let home = cpl_check::home_dir();
    type PluginsDir = Box<dyn Fn(&str) -> Option<PathBuf>>;
    let (config, plugins): (Option<PathBuf>, PluginsDir) = if cfg!(windows) {
        let roaming = var("APPDATA").map(|d| d.join("JetBrains"));
        let r = roaming.clone();
        (
            roaming,
            Box::new(move |ide| Some(r.clone()?.join(ide).join("plugins"))),
        )
    } else if cfg!(target_os = "macos") {
        let support = home.map(|h| h.join("Library/Application Support/JetBrains"));
        let s = support.clone();
        (
            support,
            Box::new(move |ide| Some(s.clone()?.join(ide).join("plugins"))),
        )
    } else {
        let config = var("XDG_CONFIG_HOME").or_else(|| Some(home.clone()?.join(".config")));
        let data = var("XDG_DATA_HOME").or_else(|| Some(home.clone()?.join(".local/share")));
        (
            config.map(|c| c.join("JetBrains")),
            Box::new(move |ide| Some(data.clone()?.join("JetBrains").join(ide))),
        )
    };
    let Some(entries) = config.and_then(|c| std::fs::read_dir(c).ok()) else {
        return Vec::new();
    };
    let mut ides: Vec<String> = entries
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|name| {
            let digits = name.trim_start_matches(|c: char| c.is_ascii_alphabetic());
            digits.len() < name.len()
                && digits.contains('.')
                && digits.chars().all(|c| c.is_ascii_digit() || c == '.')
        })
        .collect();
    ides.sort();
    ides.iter().filter_map(|ide| plugins(ide)).collect()
}

/// Unpacks the plugin in `zip`, or the edge release's, into `dir` or every
/// installed IDE's plugins folder.
fn install_jetbrains(zip: Option<PathBuf>, dir: Option<PathBuf>) -> Result<String, String> {
    let targets = match dir {
        Some(dir) => vec![dir],
        None => jetbrains_plugin_dirs(),
    };
    if targets.is_empty() {
        return Err("no JetBrains IDE found (open one once first), or pass --dir PLUGINS".into());
    }
    let downloaded =
        std::env::temp_dir().join(format!("cp-lang-jetbrains-{}.zip", std::process::id()));
    let file = match zip {
        Some(file) => file,
        None => {
            let url = format!(
                "{}/download/edge/cp-lang-jetbrains.zip",
                crate::toolchain::releases_url()
            );
            crate::toolchain::fetch(&url, &downloaded)?;
            downloaded.clone()
        }
    };
    let result = targets.iter().try_for_each(|target| unzip(&file, target));
    let _ = std::fs::remove_file(&downloaded);
    result?;
    let list: Vec<String> = targets.iter().map(|t| t.display().to_string()).collect();
    Ok(format!(
        "installed CP Lang for JetBrains in {}\nrestart the IDE; set the path to `cpl` under Settings, Tools, CP Lang if it is not on the PATH",
        list.join(", ")
    ))
}

/// `unzip`, or `tar` (bsdtar on macOS and Windows reads zips).
fn unzip(file: &Path, into: &Path) -> Result<(), String> {
    std::fs::create_dir_all(into).map_err(|e| format!("can't write {}: {e}", into.display()))?;
    let unzip = std::process::Command::new("unzip")
        .arg("-oq")
        .arg(file)
        .arg("-d")
        .arg(into)
        .output();
    let out = match unzip {
        Ok(out) => out,
        Err(_) => std::process::Command::new("tar")
            .arg("-xf")
            .arg(file)
            .arg("-C")
            .arg(into)
            .output()
            .map_err(|e| format!("unpacking the plugin needs unzip or tar: {e}"))?,
    };
    match out.status.success() {
        true => Ok(()),
        false => Err(format!(
            "could not unpack {}: {}",
            file.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        )),
    }
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

    #[test]
    fn a_neovim_older_than_0_10_gets_a_warning_with_where_to_upgrade() {
        let apt = "NVIM v0.6.1\nBuild type: Release\nLuaJIT 2.1.0-beta3\n";
        let warning = old_neovim(apt).expect("0.6 is too old");
        assert!(
            warning.contains("Neovim 0.6") && warning.contains("0.10 or later"),
            "{warning}"
        );
        assert!(
            warning.contains("github.com/neovim/neovim/releases"),
            "{warning}"
        );
        assert_eq!(old_neovim("NVIM v0.10.0\n"), None);
        assert_eq!(old_neovim("NVIM v0.11.0-dev-1234+g5678\n"), None);
        assert_eq!(old_neovim("NVIM v1.0.0\n"), None);
        assert_eq!(old_neovim("not neovim\n"), None);
    }

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
