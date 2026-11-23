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
into every installed 2024.3 or newer IDE's plugins folder; the commercial
IDEs run `cpl lsp` with their built-in client, and IntelliJ IDEA or PyCharm
Community get LSP4IJ installed beside it. Restart the IDE, then
`cpl setup jetbrains --check` reads its log to say whether CP Lang loaded.

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
            write_files(editor, &files, dir)
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
    write_all(&dir, files).map_err(|e| format!("can't write {}: {e}", dir.display()))?;
    Ok(format!(
        "installed CP Lang for {editor} in {}\nopen a .cp file to start; `cpl` must be on the PATH (or set g:cpl_path)",
        dir.display()
    ))
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
        file.display()
    ))
}

/// The plugin zip's top folder, `rootProject.name` in editors/jetbrains.
const JETBRAINS_PLUGIN: &str = "cp-lang-jetbrains";
/// The LSP4IJ that Community IDEs get, `lsp4ijVersion` in editors/jetbrains.
const LSP4IJ: &str = "0.19.4";

/// An installed JetBrains IDE: its config folder's name (`IntelliJIdea2024.3`,
/// `IdeaIC2025.1`, ...), its plugins folder and its log.
struct Ide {
    name: String,
    plugins: PathBuf,
    log: Option<PathBuf>,
}

impl Ide {
    /// IntelliJ IDEA and PyCharm Community have no built-in LSP client; the
    /// plugin reaches `cpl lsp` through LSP4IJ there.
    fn community(&self) -> bool {
        self.name.starts_with("IdeaIC") || self.name.starts_with("PyCharmCE")
    }

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
    let Some(entries) = config.and_then(|c| std::fs::read_dir(c).ok()) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|name| {
            let digits = name.trim_start_matches(|c: char| c.is_ascii_alphabetic());
            digits.len() < name.len()
                && digits.contains('.')
                && digits.chars().all(|c| c.is_ascii_digit() || c == '.')
        })
        .collect();
    names.sort();
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

/// Unpacks the plugin in `zip`, or the edge release's, into `dir` or every
/// installed IDE's plugins folder, with LSP4IJ for a Community IDE, then
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
        return Err("no JetBrains IDE found (open one once first), or pass --dir PLUGINS".into());
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
    let community: Vec<&Ide> = ides
        .iter()
        .filter(|ide| ide.community() && !ide.plugins.join("lsp4ij/lib").is_dir())
        .collect();
    if !community.is_empty() {
        let lsp4ij = std::env::temp_dir().join(format!("lsp4ij-{}.zip", std::process::id()));
        let url = format!(
            "https://plugins.jetbrains.com/plugin/download?pluginId=com.redhat.devtools.lsp4ij&version={LSP4IJ}"
        );
        let result = crate::toolchain::fetch(&url, &lsp4ij).and_then(|()| {
            community
                .iter()
                .try_for_each(|ide| unzip(&lsp4ij, &ide.plugins))
        });
        let _ = std::fs::remove_file(&lsp4ij);
        result.map_err(|e| {
            format!("Community IDEs need LSP4IJ for `cpl lsp`, and installing it failed: {e}")
        })?;
    }
    let mut report = String::new();
    for ide in &ides {
        let missing = missing_jetbrains_files(ide);
        if !missing.is_empty() {
            return Err(format!(
                "the plugin did not install in {}: no {}",
                ide.plugins.display(),
                missing.join(", ")
            ));
        }
        let name = if ide.name.is_empty() {
            "the IDE"
        } else {
            &ide.name
        };
        let lsp = if ide.community() {
            ", with LSP4IJ for `cpl lsp`"
        } else {
            ""
        };
        report += &format!(
            "installed CP Lang for {name} in {}{lsp}\n",
            ide.plugins.display()
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

/// What `ide` lacks to load the plugin: its jar, and LSP4IJ in a Community IDE.
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
    if ide.community() && !has_jar(ide.plugins.join("lsp4ij/lib")) {
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
        return Ok(ours.unwrap_or("CP Lang").trim().to_string());
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

    /// The plugin folder and LSP4IJ version are the ones editors/jetbrains builds with.
    #[test]
    fn jetbrains_names_match_the_gradle_build() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../editors/jetbrains");
        let settings = std::fs::read_to_string(root.join("settings.gradle.kts")).unwrap();
        assert!(settings.contains(&format!("rootProject.name = \"{JETBRAINS_PLUGIN}\"")));
        let properties = std::fs::read_to_string(root.join("gradle.properties")).unwrap();
        assert!(properties.contains(&format!("lsp4ijVersion = {LSP4IJ}\n")));
    }

    #[test]
    fn the_ide_log_says_whether_cp_lang_loaded_at_the_last_start() {
        let started = "INFO - #c.i.i.i.AppStarter - ------ IDE STARTED ------\n";
        let loaded = "INFO - #c.i.i.p.PluginManager - Loaded custom plugins: LSP4IJ (0.19.4), CP Lang (0.3.0)\n";
        let refused = "WARN - #c.i.i.p.PluginManager - Plugin 'CP Lang' requires plugin 'com.intellij.modules.ultimate' to be installed\n";
        assert_eq!(
            plugin_load(&format!("{started}{loaded}")),
            Ok("CP Lang (0.3.0)".into())
        );
        let failed = plugin_load(&format!("{started}{loaded}{started}{refused}")).unwrap_err();
        assert!(failed.contains("com.intellij.modules.ultimate"), "{failed}");
        assert!(plugin_load(started).is_err());
    }

    #[test]
    fn community_and_old_ides_are_told_apart() {
        let ide = |name: &str| Ide {
            name: name.into(),
            plugins: PathBuf::new(),
            log: None,
        };
        assert!(ide("IdeaIC2024.3").community() && ide("PyCharmCE2025.1").community());
        assert!(!ide("IntelliJIdea2024.3").community() && !ide("CLion2025.1").community());
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
