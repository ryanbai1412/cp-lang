//! `cpl setup <editor>`: writes the editor files built into this binary
//! where the editor loads them, so one command sets an editor up.

use cpl_diag::Shown;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) const USAGE: &str = "\
usage: cpl setup <neovim | vim | vscode | jetbrains> [OPTIONS]

Writes CP Lang's editor files where the editor loads them at startup, then
says where. Neovim gets the filetype, highlighting and `cpl lsp`:
diagnostics, completion, `gd` to the definition and formatting on save.
Vim gets the filetype and highlighting, and `cpl lsp` through vim-lsp,
which it installs beside CP Lang unless vim-lsp or yegappan/lsp is already
there. VS Code gets the extension from the
edge release, installed with `code --install-extension` (`$CPL_CODE` names
another editor command, such as `codium` or `cursor`). Run it again after
`cpl self update`. JetBrains IDEs get the edge release's plugin unpacked
into every installed 2024.3 or newer IDE's plugins folder, with LSP4IJ
beside it to run `cpl lsp` (the free IntelliJ IDEA has no built-in LSP
client). Restart the IDE, then `cpl setup jetbrains --check` reads its log
to say whether CP Lang and LSP4IJ loaded.

options:
  --dir DIR     write into DIR instead (a `pack/*/start/<name>` folder)
  --vsix FILE   install this extension file instead of downloading it
  --zip FILE    install this JetBrains plugin file instead of downloading it
  --check       JetBrains: say whether each IDE loaded the plugin when it
                last started
  --json        print one JSON object: the `editor`, `ok`, and the `message`
                (or the `error`) the text output gives
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
    let mut check = false;
    let mut json = false;
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
            "--check" => check = true,
            "--json" => json = true,
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
            "name the editor: `cpl setup neovim`, `vim`, `vscode` or `jetbrains`",
        );
    };
    let editor = match editor {
        "code" => "vscode",
        "nvim" => "neovim",
        "intellij" | "idea" => "jetbrains",
        other => other,
    };
    let outcome = match editor {
        "vscode" => install_vscode(vsix),
        "jetbrains" if check => check_jetbrains(),
        "jetbrains" => install_jetbrains(zip, dir.take()),
        _ => {
            let Some(files) = files(editor) else {
                return usage(
                    stderr,
                    &format!("no setup for `{editor}`: `neovim`, `vim`, `vscode` or `jetbrains`"),
                );
            };
            let dir = dir.or_else(|| package_dir(editor));
            let written = write_files(editor, &files, dir.clone());
            match (editor, dir) {
                ("vim", Some(dir)) => written.and_then(|done| Ok(done + &install_vim_lsp(&dir)?)),
                _ => written,
            }
        }
    };
    let ok = outcome.is_ok();
    if json {
        let (key, said) = match &outcome {
            Ok(done) => ("message", done),
            Err(e) => ("error", e),
        };
        let _ = writeln!(
            stdout,
            "{{\"editor\":{},\"ok\":{ok},\"{key}\":{}}}",
            cpl_json::escape_string(editor),
            cpl_json::escape_string(said.trim_end())
        );
    } else {
        match outcome {
            Ok(done) => {
                let _ = writeln!(stdout, "{done}");
            }
            Err(e) => *stderr += &format!("cpl: {e}\n"),
        }
    }
    if ok && editor == "neovim" {
        let version = std::process::Command::new("nvim").arg("--version").output();
        if let Some(warning) = version
            .ok()
            .and_then(|out| old_neovim(&String::from_utf8_lossy(&out.stdout)))
        {
            *stderr += &warning;
        }
    }
    u8::from(!ok)
}

/// Writes Neovim's or Vim's `files` into `dir` (by default where the editor
/// loads packages), and says where.
fn write_files(
    editor: &str,
    files: &[(&str, &str)],
    dir: Option<PathBuf>,
) -> Result<String, String> {
    let dir = dir
        .or_else(|| package_dir(editor))
        .ok_or("no home folder to install into; pass --dir")?;
    write_all(&dir, files).map_err(|e| format!("can't write {}: {e}", dir.shown()))?;
    Ok(format!(
        "installed CP Lang for {editor} in {}\nopen a .cp file to start; `cpl` must be on the PATH (or set g:cpl_path)",
        dir.shown()
    ))
}

/// The vim-lsp commit `cpl setup vim` installs: Vim has no LSP client of its
/// own, and vim-lsp runs on the Vim 8.2 the package supports.
const VIM_LSP: &str = "977e704cdbbbbc89a54f8f226d3468b0aa0405c9";

/// Installs vim-lsp beside CP Lang's `package` (`pack/cpl/start/cplang`)
/// unless Vim already has a client, and says which client runs `cpl lsp`.
fn install_vim_lsp(package: &Path) -> Result<String, String> {
    let start = package
        .parent()
        .ok_or("no folder beside the package for vim-lsp")?;
    if let Some(client) = package.ancestors().nth(3).and_then(vim_lsp_client) {
        return Ok(format!(
            "\n`cpl lsp` runs through the {client} already installed"
        ));
    }
    let dest = start.join("vim-lsp");
    let id = std::process::id();
    let zip = std::env::temp_dir().join(format!("vim-lsp-{id}.zip"));
    let unpacked = start.join(format!(".vim-lsp-{id}"));
    let url = format!("https://github.com/prabirshrestha/vim-lsp/archive/{VIM_LSP}.zip");
    let result = crate::toolchain::fetch(&url, &zip)
        .and_then(|()| unzip(&zip, &unpacked))
        .and_then(|()| {
            let _ = std::fs::remove_dir_all(&dest);
            std::fs::rename(unpacked.join(format!("vim-lsp-{VIM_LSP}")), &dest)
                .map_err(|e| format!("can't write {}: {e}", dest.shown()))
        });
    let _ = std::fs::remove_file(&zip);
    let _ = std::fs::remove_dir_all(&unpacked);
    match result {
        Ok(()) => Ok(format!(
            "\nwith vim-lsp in {} for `cpl lsp`: diagnostics, completion, hover and go to definition",
            dest.shown()
        )),
        Err(e) => Err(format!(
            "CP Lang's highlighting is installed, but vim-lsp for `cpl lsp` is not: {e}\n\
             to install it by hand: git clone https://github.com/prabirshrestha/vim-lsp {}",
            dest.shown()
        )),
    }
}

/// The LSP client a Vim whose folder is `vim` (`~/.vim`) already loads, from
/// its packages, vim-plug's `plugged/` or pathogen's `bundle/`, other than the
/// vim-lsp `cpl setup vim` installed (that one it refreshes).
fn vim_lsp_client(vim: &Path) -> Option<&'static str> {
    let dirs = |dir: PathBuf| {
        std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>()
    };
    let mut plugins: Vec<PathBuf> = ["plugged", "bundle"]
        .iter()
        .flat_map(|dir| dirs(vim.join(dir)))
        .collect();
    for pack in dirs(vim.join("pack")) {
        if pack.ends_with("cpl") {
            continue;
        }
        for kind in ["start", "opt"] {
            plugins.extend(dirs(pack.join(kind)));
        }
    }
    plugins.iter().find_map(|plugin| {
        if plugin.join("autoload/lsp.vim").is_file() {
            Some("vim-lsp")
        } else if plugin.join("autoload/lsp/lsp.vim").is_file() {
            Some("yegappan/lsp")
        } else {
            None
        }
    })
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
    let listed = std::process::Command::new(&code)
        .args(["--list-extensions", "--show-versions"])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
        .unwrap_or_default();
    let Some(installed) = listed
        .lines()
        .find(|l| l.to_ascii_lowercase().starts_with("cp-lang.cp-lang@"))
    else {
        return Err(format!(
            "`{code} --install-extension` succeeded, but `{code} --list-extensions` does not list cp-lang.cp-lang"
        ));
    };
    Ok(format!(
        "installed CP Lang for VS Code ({}) from {}\nopen a .cp file to start; `cpl` must be on the PATH (or set `cp.server.path`)",
        installed.trim(),
        file.shown()
    ))
}

/// The plugin zip's top folder, `rootProject.name` in editors/jetbrains.
const JETBRAINS_PLUGIN: &str = "cp-lang-jetbrains";
/// The LSP4IJ every IDE gets, `lsp4ijVersion` in editors/jetbrains.
const LSP4IJ: &str = "0.30.1";

/// An installed JetBrains IDE: its config folder's name (`IntelliJIdea2024.3`,
/// `IdeaIC2025.1`, ...), its plugins folder and its log.
struct Ide {
    name: String,
    plugins: PathBuf,
    log: Option<PathBuf>,
}

impl Ide {
    /// The plugin's `sinceBuild` is 243, the 2024.3 releases.
    fn too_old(&self) -> bool {
        let digits = self
            .name
            .trim_start_matches(|c: char| c.is_ascii_alphabetic());
        let mut parts = digits.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
        let version = (parts.next().unwrap_or(0), parts.next().unwrap_or(0));
        !digits.is_empty() && version < (2024, 3)
    }
}

/// Each installed JetBrains IDE, from its config folders.
fn jetbrains_ides() -> Vec<Ide> {
    let var = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let home = cpl_check::home_dir();
    let fresh = never_started_ides(home.as_deref(), &var);
    type Folder = Box<dyn Fn(&str) -> Option<PathBuf>>;
    let (config, plugins, log): (Option<PathBuf>, Folder, Folder) = if cfg!(windows) {
        let roaming = var("APPDATA").map(|d| d.join("JetBrains"));
        let local = var("LOCALAPPDATA").map(|d| d.join("JetBrains"));
        let r = roaming.clone();
        (
            roaming,
            Box::new(move |ide| Some(r.clone()?.join(ide).join("plugins"))),
            Box::new(move |ide| Some(local.clone()?.join(ide).join("log/idea.log"))),
        )
    } else if cfg!(target_os = "macos") {
        let support = home
            .clone()
            .map(|h| h.join("Library/Application Support/JetBrains"));
        let logs = home.map(|h| h.join("Library/Logs/JetBrains"));
        let s = support.clone();
        (
            support,
            Box::new(move |ide| Some(s.clone()?.join(ide).join("plugins"))),
            Box::new(move |ide| Some(logs.clone()?.join(ide).join("idea.log"))),
        )
    } else {
        let config = var("XDG_CONFIG_HOME").or_else(|| Some(home.clone()?.join(".config")));
        let data = var("XDG_DATA_HOME").or_else(|| Some(home.clone()?.join(".local/share")));
        let cache = var("XDG_CACHE_HOME").or_else(|| Some(home.clone()?.join(".cache")));
        (
            config.map(|c| c.join("JetBrains")),
            Box::new(move |ide| Some(data.clone()?.join("JetBrains").join(ide))),
            Box::new(move |ide| {
                Some(
                    cache
                        .clone()?
                        .join("JetBrains")
                        .join(ide)
                        .join("log/idea.log"),
                )
            }),
        )
    };
    let mut names: Vec<String> = config
        .and_then(|c| std::fs::read_dir(c).ok())
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .chain(fresh)
        .filter(|name| {
            let digits = name.trim_start_matches(|c: char| c.is_ascii_alphabetic());
            digits.len() < name.len()
                && digits.contains('.')
                && digits.chars().all(|c| c.is_ascii_digit() || c == '.')
        })
        .collect();
    names.sort();
    names.dedup();
    names
        .into_iter()
        .filter_map(|name| {
            Some(Ide {
                plugins: plugins(&name)?,
                log: log(&name),
                name,
            })
        })
        .collect()
}

/// The folder names (`IntelliJIdea2025.3`) of the IDEs installed where
/// JetBrains, Toolbox, snap or an unpacked archive put them, or whose launcher
/// is on the PATH. An IDE gets its config folder only when it first starts,
/// and its `product-info.json` names it before then.
fn never_started_ides(home: Option<&Path>, var: &dyn Fn(&str) -> Option<PathBuf>) -> Vec<String> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            for launcher in [
                "idea",
                "idea.sh",
                "idea64.exe",
                "pycharm",
                "pycharm.sh",
                "pycharm64.exe",
                "clion",
                "clion.sh",
                "clion64.exe",
            ] {
                let Ok(real) = std::fs::canonicalize(dir.join(launcher)) else {
                    continue;
                };
                // `<install>/bin/idea.sh`, or `X.app/Contents/MacOS/idea`.
                if let Some(install) = real.parent().and_then(Path::parent) {
                    roots.push(install.to_path_buf());
                }
            }
        }
    }
    let mut parents: Vec<PathBuf> = Vec::new();
    if cfg!(windows) {
        parents.extend(var("ProgramFiles").map(|d| d.join("JetBrains")));
        if let Some(local) = var("LOCALAPPDATA") {
            parents.push(local.join("Programs"));
            parents.push(local.join("JetBrains/Toolbox/apps"));
        }
    } else if cfg!(target_os = "macos") {
        parents.push("/Applications".into());
        parents.extend(home.map(|h| h.join("Applications")));
    } else {
        parents.extend(["/opt".into(), "/usr/local".into(), "/snap".into()]);
        if let Some(home) = home {
            parents.push(home.to_path_buf());
            parents.push(home.join("Downloads"));
            parents.push(home.join(".local/share/JetBrains/Toolbox/apps"));
        }
    }
    let children = |dir: &Path| -> Vec<PathBuf> {
        std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .filter_map(|e| Some(e.ok()?.path()))
            .filter(|p| p.is_dir())
            .collect()
    };
    for parent in &parents {
        for child in children(parent) {
            // Toolbox and snap keep each version a level down.
            roots.extend(
                children(&child)
                    .into_iter()
                    .chain(Some(child.join("current"))),
            );
            roots.push(child);
        }
    }
    roots
        .iter()
        .filter_map(|root| {
            let info = [
                root.join("product-info.json"),
                root.join("Contents/Resources/product-info.json"),
            ]
            .into_iter()
            .find_map(|f| std::fs::read_to_string(f).ok())?;
            let info = cpl_json::parse(&info).ok()?;
            Some(info.get("dataDirectoryName")?.as_str()?.to_string())
        })
        .collect()
}

/// Unpacks the plugin in `zip`, or the edge release's, into `dir` or every
/// installed IDE's plugins folder, with LSP4IJ, then
/// checks that each IDE has what it needs to load it.
fn install_jetbrains(zip: Option<PathBuf>, dir: Option<PathBuf>) -> Result<String, String> {
    let ides = match dir {
        Some(dir) => vec![Ide {
            name: String::new(),
            plugins: dir,
            log: None,
        }],
        None => jetbrains_ides(),
    };
    if ides.is_empty() {
        return Err("no JetBrains IDE found: none has started yet, has its launcher (`idea`, `idea.sh`) on the PATH, or is in a usual install folder; pass --dir PLUGINS, or start the IDE once".into());
    }
    let (old, ides): (Vec<Ide>, Vec<Ide>) = ides.into_iter().partition(Ide::too_old);
    let old: Vec<&str> = old.iter().map(|ide| ide.name.as_str()).collect();
    if ides.is_empty() {
        return Err(format!(
            "CP Lang needs a 2024.3 or newer JetBrains IDE; found only {}",
            old.join(", ")
        ));
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
    let result = ides.iter().try_for_each(|ide| unzip(&file, &ide.plugins));
    let _ = std::fs::remove_file(&downloaded);
    result?;
    // The plugin runs `cpl lsp` through LSP4IJ wherever it is loaded: the free
    // IntelliJ IDEA has the built-in client's classes but not its module.
    let stale: Vec<&Ide> = ides
        .iter()
        .filter(|ide| lsp4ij_version(&ide.plugins).is_none_or(|v| v < version(LSP4IJ)))
        .collect();
    if !stale.is_empty() {
        let lsp4ij = std::env::temp_dir().join(format!("lsp4ij-{}.zip", std::process::id()));
        let url = format!(
            "https://plugins.jetbrains.com/plugin/download?pluginId=com.redhat.devtools.lsp4ij&version={LSP4IJ}"
        );
        let result = crate::toolchain::fetch(&url, &lsp4ij).and_then(|()| {
            stale.iter().try_for_each(|ide| {
                // An older LSP4IJ's jars would load beside the new ones.
                let _ = std::fs::remove_dir_all(ide.plugins.join("lsp4ij"));
                unzip(&lsp4ij, &ide.plugins)
            })
        });
        let _ = std::fs::remove_file(&lsp4ij);
        result.map_err(|e| {
            format!("CP Lang needs LSP4IJ for `cpl lsp`, and installing it failed: {e}")
        })?;
    }
    let mut report = String::new();
    for ide in &ides {
        let missing = missing_jetbrains_files(ide);
        if !missing.is_empty() {
            return Err(format!(
                "the plugin did not install in {}: no {}",
                ide.plugins.shown(),
                missing.join(", ")
            ));
        }
        let name = if ide.name.is_empty() {
            "the IDE"
        } else {
            &ide.name
        };
        report += &format!(
            "installed CP Lang for {name} in {}, with LSP4IJ for `cpl lsp`\n",
            ide.plugins.shown()
        );
    }
    if !old.is_empty() {
        report += &format!(
            "skipped {}: CP Lang needs 2024.3 or newer\n",
            old.join(", ")
        );
    }
    report += "restart the IDE, then `cpl setup jetbrains --check` says whether CP Lang loaded";
    Ok(report)
}

/// What `ide` lacks to load the plugin and run `cpl lsp`: its jar and LSP4IJ's.
fn missing_jetbrains_files(ide: &Ide) -> Vec<String> {
    let has_jar = |dir: PathBuf| {
        std::fs::read_dir(dir).is_ok_and(|entries| {
            entries
                .filter_map(Result::ok)
                .any(|e| e.path().extension().is_some_and(|x| x == "jar"))
        })
    };
    let mut missing = Vec::new();
    if !has_jar(ide.plugins.join(JETBRAINS_PLUGIN).join("lib")) {
        missing.push(format!("{JETBRAINS_PLUGIN}/lib/*.jar"));
    }
    if !has_jar(ide.plugins.join("lsp4ij/lib")) {
        missing.push("lsp4ij/lib/*.jar".into());
    }
    missing
}

/// `cpl setup jetbrains --check`: whether each IDE with the plugin loaded it
/// at its last start, from the IDE's own log.
fn check_jetbrains() -> Result<String, String> {
    let ides: Vec<Ide> = jetbrains_ides()
        .into_iter()
        .filter(|ide| ide.plugins.join(JETBRAINS_PLUGIN).is_dir())
        .collect();
    if ides.is_empty() {
        return Err("no JetBrains IDE has CP Lang; run `cpl setup jetbrains`".into());
    }
    let mut report = String::new();
    let mut failed = Vec::new();
    for ide in &ides {
        let log = ide
            .log
            .as_ref()
            .and_then(|log| std::fs::read_to_string(log).ok());
        match log.as_deref().map(plugin_load) {
            Some(Ok(line)) => report += &format!("{}: CP Lang loaded ({line})\n", ide.name),
            Some(Err(why)) => failed.push(format!("{}: {why}", ide.name)),
            None => failed.push(format!(
                "{}: no log yet; start the IDE once, then check again",
                ide.name
            )),
        }
        let missing = missing_jetbrains_files(ide);
        if !missing.is_empty() {
            failed.push(format!(
                "{}: no {}; run `cpl setup jetbrains`",
                ide.name,
                missing.join(", ")
            ));
        }
    }
    match failed.is_empty() {
        true => Ok(report.trim_end().to_string()),
        false => Err(format!("{report}{}", failed.join("\n"))),
    }
}

/// From an IDE's log, its last start's verdict on CP Lang: the plugin list
/// that names it, or why it did not load.
fn plugin_load(log: &str) -> Result<String, String> {
    let last = log.rfind("IDE STARTED").map_or(log, |at| &log[at..]);
    if let Some(line) = last
        .lines()
        .find(|l| l.contains("Loaded custom plugins:") && l.contains("CP Lang"))
    {
        let names = line.split("Loaded custom plugins:").nth(1).unwrap_or("");
        let ours = names.split(", ").find(|p| p.contains("CP Lang"));
        let ours = ours.unwrap_or("CP Lang").trim().to_string();
        if !names.contains("LSP4IJ") {
            return Err(format!(
                "{ours} loaded without LSP4IJ, so .cp files get highlighting but no `cpl lsp`; run `cpl setup jetbrains` again"
            ));
        }
        return Ok(ours);
    }
    match last
        .lines()
        .find(|l| l.contains("CP Lang") || l.contains("dev.cplang.ide"))
    {
        Some(line) => Err(format!(
            "CP Lang did not load at the last start: {}",
            line.trim()
        )),
        None => {
            Err("CP Lang did not load at the last start; restart the IDE and check again".into())
        }
    }
}

/// The LSP4IJ in `plugins`, from its jar's name (`lsp4ij/lib/lsp4ij-0.30.1.jar`).
fn lsp4ij_version(plugins: &Path) -> Option<Vec<u32>> {
    std::fs::read_dir(plugins.join("lsp4ij/lib"))
        .ok()?
        .filter_map(Result::ok)
        .find_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            let v = name.strip_prefix("lsp4ij-")?.strip_suffix(".jar")?;
            v.chars()
                .all(|c| c.is_ascii_digit() || c == '.')
                .then(|| version(v))
        })
}

fn version(text: &str) -> Vec<u32> {
    text.split('.').map(|p| p.parse().unwrap_or(0)).collect()
}

/// `unzip`, or `tar` (bsdtar on macOS and Windows reads zips).
fn unzip(file: &Path, into: &Path) -> Result<(), String> {
    std::fs::create_dir_all(into).map_err(|e| format!("can't write {}: {e}", into.shown()))?;
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
            file.shown(),
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
    fn vim_keeps_the_lsp_client_it_has_and_otherwise_gets_vim_lsp() {
        let vim = std::env::temp_dir().join(format!("cpl-vim-{}", std::process::id()));
        let plugin = |dir: &str, file: &str| {
            let path = vim.join(dir).join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "").unwrap();
        };
        plugin("pack/cpl/start/vim-lsp", "autoload/lsp.vim");
        plugin("pack/colors/start/gruvbox", "autoload/gruvbox.vim");
        assert_eq!(vim_lsp_client(&vim), None, "its own vim-lsp is refreshed");
        plugin("pack/plugins/opt/lsp", "autoload/lsp/lsp.vim");
        assert_eq!(vim_lsp_client(&vim), Some("yegappan/lsp"));
        std::fs::remove_dir_all(vim.join("pack/plugins")).unwrap();
        plugin("plugged/vim-lsp", "autoload/lsp.vim");
        assert_eq!(vim_lsp_client(&vim), Some("vim-lsp"));
        std::fs::remove_dir_all(&vim).unwrap();
    }

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

    /// The plugin folder and LSP4IJ version are the ones editors/jetbrains builds with.
    #[test]
    fn jetbrains_names_match_the_gradle_build() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../editors/jetbrains");
        let settings = std::fs::read_to_string(root.join("settings.gradle.kts")).unwrap();
        assert!(settings.contains(&format!("rootProject.name = \"{JETBRAINS_PLUGIN}\"")));
        let properties = std::fs::read_to_string(root.join("gradle.properties")).unwrap();
        // `lines()`, not `\n`: a Windows checkout has CRLF.
        let want = format!("lsp4ijVersion = {LSP4IJ}");
        assert!(
            properties.lines().any(|line| line.trim_end() == want),
            "{properties}"
        );
    }

    #[test]
    fn the_ide_log_says_whether_cp_lang_loaded_at_the_last_start() {
        let started = "INFO - #c.i.i.i.AppStarter - ------ IDE STARTED ------\n";
        let loaded = "INFO - #c.i.i.p.PluginManager - Loaded custom plugins: LSP4IJ (0.30.1), CP Lang (0.3.0)\n";
        let refused = "WARN - #c.i.i.p.PluginManager - Plugin 'CP Lang' requires plugin 'com.intellij.modules.ultimate' to be installed\n";
        assert_eq!(
            plugin_load(&format!("{started}{loaded}")),
            Ok("CP Lang (0.3.0)".into())
        );
        let failed = plugin_load(&format!("{started}{loaded}{started}{refused}")).unwrap_err();
        assert!(failed.contains("com.intellij.modules.ultimate"), "{failed}");
        assert!(plugin_load(started).is_err());
        // The free IntelliJ IDEA has no built-in client: without LSP4IJ it only highlights.
        let alone = "INFO - #c.i.i.p.PluginManager - Loaded custom plugins: CP Lang (0.3.0)\n";
        let failed = plugin_load(&format!("{started}{alone}")).unwrap_err();
        assert!(failed.contains("without LSP4IJ"), "{failed}");
    }

    /// An IDE whose LSP4IJ is older than the pinned one gets the pinned one.
    #[test]
    fn the_installed_lsp4ij_version_comes_from_its_jar() {
        let plugins = std::env::temp_dir().join(format!("cpl-lsp4ij-{}", std::process::id()));
        let lib = plugins.join("lsp4ij/lib");
        std::fs::create_dir_all(&lib).unwrap();
        assert_eq!(lsp4ij_version(&plugins), None);
        for jar in [
            "lsp4ij-0.19.4-searchableOptions.jar",
            "gson-2.10.jar",
            "lsp4ij-0.19.4.jar",
        ] {
            std::fs::write(lib.join(jar), "").unwrap();
        }
        assert_eq!(lsp4ij_version(&plugins), Some(vec![0, 19, 4]));
        assert!(lsp4ij_version(&plugins).unwrap() < version(LSP4IJ));
        assert!(version("0.30.1") > version("0.4.10") && version("1.0") > version("0.30.1"));
        std::fs::remove_dir_all(&plugins).unwrap();
    }

    #[test]
    fn old_ides_are_told_apart() {
        let ide = |name: &str| Ide {
            name: name.into(),
            plugins: PathBuf::new(),
            log: None,
        };
        assert!(ide("IntelliJIdea2024.2").too_old() && ide("IdeaIC2023.3").too_old());
        assert!(
            !ide("IdeaIC2024.3").too_old() && !ide("CLion2025.1").too_old() && !ide("").too_old()
        );
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
