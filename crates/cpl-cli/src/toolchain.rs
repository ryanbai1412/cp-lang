//! Installed toolchains: the launcher at `$CPL_HOME/bin/cpl` and `cpl self`.
//!
//! `$CPL_HOME` (default `~/.cpl`) holds `bin/cpl`, the launcher (a copy of
//! a toolchain's own binary), `toolchains/<id>/` with each installed
//! toolchain and the `release.json` it came from, and `settings.toml` with
//! the default selector, each channel's current toolchain and one rollback
//! point. Downloads go through `curl` and archives through `tar`, both
//! present on every supported system (decisions.md).

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{sha256, EXIT_ERRORS, EXIT_USAGE};

pub const RELEASES: &str = "https://github.com/ryanbai1412/cp-lang/releases";
const EXE: &str = std::env::consts::EXE_SUFFIX;

pub const SELF_USAGE: &str = "\
usage: cpl self <command>

commands:
  update [channel]     install the newest build of the default channel (or
                       the one named), keeping the current one for rollback
  channel [toolchain]  show or set the default: stable, edge, a version like
                       0.3.0, or a dated edge build like edge-2026-11-22
  rollback             go back to the toolchain used before the last update
                       or channel change; works offline
  install <toolchain>  download a toolchain now, for use offline later
  list                 list installed toolchains

`cpl +<toolchain> <command>` runs one command with another toolchain. The
toolchain a command uses is the first of: `+<toolchain>`, $CPL_VERSION,
`version = \"...\"` or `channel = \"...\"` in a cpl.toml in the current folder
or above, the default.
";

/// What names a toolchain on the command line, in `$CPL_VERSION`, in a
/// `cpl.toml` or as the default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    /// `stable` or `edge`: the channel's current toolchain.
    Channel(&'static str),
    /// `0.3.0`: a stable release.
    Version(String),
    /// `edge-2026-11-22`: the first edge build of that day.
    Dated(String),
    /// `edge-2026-11-22-1a2b3c4d`: one installed edge build.
    Toolchain(String),
}

impl Selector {
    pub fn parse(text: &str) -> Result<Selector, String> {
        let text = text.trim();
        match text {
            "stable" => return Ok(Selector::Channel("stable")),
            "edge" => return Ok(Selector::Channel("edge")),
            _ => {}
        }
        let version = text.strip_prefix('v').unwrap_or(text);
        if is_version(version) {
            return Ok(Selector::Version(version.to_string()));
        }
        if let Some(rest) = text.strip_prefix("edge-") {
            if is_date(rest) {
                return Ok(Selector::Dated(text.to_string()));
            }
            if let Some((date, commit)) = rest.split_at_checked(10) {
                if is_date(date)
                    && commit.len() > 1
                    && commit[1..].bytes().all(|b| b.is_ascii_hexdigit())
                    && commit.starts_with('-')
                {
                    return Ok(Selector::Toolchain(text.to_string()));
                }
            }
        }
        Err(format!("`{text}` is not a toolchain: use stable, edge, a version like 0.3.0, or edge-YYYY-MM-DD"))
    }

    /// The release whose `latest.json` describes this toolchain; `None` is
    /// the newest stable release.
    fn tag(&self) -> Result<Option<String>, String> {
        match self {
            Selector::Channel("stable") => Ok(None),
            Selector::Channel(channel) => Ok(Some(channel.to_string())),
            Selector::Version(version) => Ok(Some(format!("v{version}"))),
            Selector::Dated(tag) => Ok(Some(tag.clone())),
            Selector::Toolchain(id) => Err(format!(
                "{id} is not installed, and single edge builds can't be downloaded; use edge or {}",
                &id[..15]
            )),
        }
    }
}

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Selector::Channel(name) => f.write_str(name),
            Selector::Version(text) | Selector::Dated(text) | Selector::Toolchain(text) => {
                f.write_str(text)
            }
        }
    }
}

fn is_version(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

fn is_date(text: &str) -> bool {
    text.len() == 10
        && text.bytes().enumerate().all(|(i, b)| {
            if i == 4 || i == 7 {
                b == b'-'
            } else {
                b.is_ascii_digit()
            }
        })
}

/// `key = "value"` lines before the first `[section]`; `#` starts a comment.
fn top_level_keys(text: &str) -> Vec<(String, String)> {
    let mut keys = Vec::new();
    for line in text.lines() {
        let line = line.split_once(" #").map_or(line, |(code, _)| code).trim();
        if line.starts_with('[') {
            break;
        }
        if line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .unwrap_or(value);
            keys.push((key.trim().to_string(), value.to_string()));
        }
    }
    keys
}

/// `$CPL_HOME/settings.toml`.
#[derive(Debug, Default)]
struct Settings(BTreeMap<String, String>);

impl Settings {
    fn load(home: &Path) -> Settings {
        let text = std::fs::read_to_string(home.join("settings.toml")).unwrap_or_default();
        Settings(top_level_keys(&text).into_iter().collect())
    }

    fn save(&self, home: &Path) -> Result<(), String> {
        let mut text = String::new();
        for (key, value) in &self.0 {
            text += &format!("{key} = \"{value}\"\n");
        }
        write_atomically(&home.join("settings.toml"), text.as_bytes())
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    fn set(&mut self, key: &str, value: Option<&str>) {
        match value {
            Some(value) => self.0.insert(key.to_string(), value.to_string()),
            None => self.0.remove(key),
        };
    }

    fn default_selector(&self) -> Result<Selector, String> {
        Selector::parse(self.get("default").unwrap_or("stable"))
            .map_err(|e| format!("{e} (the default in settings.toml)"))
    }

    /// Remembers the default and `channel`'s toolchain, for `cpl self rollback`.
    fn checkpoint(&mut self, channel: Option<&str>) {
        let default = self.get("default").unwrap_or("stable").to_string();
        let toolchain = channel.and_then(|c| self.get(c)).map(str::to_string);
        self.set("rollback-default", Some(&default));
        self.set("rollback-channel", channel);
        self.set("rollback-toolchain", toolchain.as_deref());
    }
}

fn write_atomically(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension(format!("tmp{}", std::process::id()));
    std::fs::write(&tmp, bytes)
        .and_then(|()| std::fs::rename(&tmp, path))
        .map_err(|e| format!("could not write {}: {e}", path.display()))
}

pub fn home() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("CPL_HOME").filter(|h| !h.is_empty()) {
        return Some(PathBuf::from(home));
    }
    let user = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(user).join(".cpl"))
}

fn toolchain_dir(home: &Path, id: &str) -> PathBuf {
    home.join("toolchains").join(id)
}

fn binary(dir: &Path) -> PathBuf {
    dir.join("bin").join(format!("cpl{EXE}"))
}

fn is_installed(home: &Path, id: &str) -> bool {
    binary(&toolchain_dir(home, id)).is_file()
}

fn installed_ids(home: &Path) -> Vec<String> {
    let mut ids: Vec<String> = std::fs::read_dir(home.join("toolchains"))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|id| is_installed(home, id))
        .collect();
    ids.sort();
    ids
}

/// The release target whose build runs here. Linux builds are static (musl),
/// and Windows on Arm runs the x86-64 build.
fn host_target() -> Option<&'static str> {
    Some(match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-musl",
        ("linux", "aarch64") => "aarch64-unknown-linux-musl",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("windows", "x86_64" | "aarch64") => "x86_64-pc-windows-msvc",
        _ => return None,
    })
}

/// One release's `latest.json`, written by scripts/release/manifest.py.
#[derive(Debug)]
struct Release {
    toolchain: String,
    tags: Vec<String>,
    assets: Vec<(String, String, String)>,
}

fn parse_release(text: &str) -> Result<Release, String> {
    use cpl_json::Value;
    let bad = |what: &str| format!("the release manifest is malformed ({what})");
    let json = cpl_json::parse(text).map_err(|e| bad(&e.to_string()))?;
    let toolchain = json
        .at(&["toolchain"])
        .as_str()
        .ok_or_else(|| bad("no toolchain"))?
        .to_string();
    let safe = |s: &str| {
        !s.is_empty()
            && !s.starts_with('.')
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    };
    if !safe(&toolchain) {
        return Err(bad("toolchain id"));
    }
    let tags = json
        .at(&["tags"])
        .as_array()
        .unwrap_or_default()
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect();
    let Value::Object(members) = json.at(&["assets"]) else {
        return Err(bad("no assets"));
    };
    let mut assets = Vec::new();
    for (target, asset) in members {
        let (Some(file), Some(sha)) =
            (asset.at(&["file"]).as_str(), asset.at(&["sha256"]).as_str())
        else {
            return Err(bad(target));
        };
        if !safe(file) {
            return Err(bad("asset file name"));
        }
        assets.push((target.clone(), file.to_string(), sha.to_ascii_lowercase()));
    }
    Ok(Release {
        toolchain,
        tags,
        assets,
    })
}

fn release_tags(home: &Path, id: &str) -> Vec<String> {
    std::fs::read_to_string(toolchain_dir(home, id).join("release.json"))
        .ok()
        .and_then(|text| parse_release(&text).ok())
        .map(|release| release.tags)
        .unwrap_or_default()
}

fn releases_url() -> String {
    std::env::var("CPL_RELEASES_URL")
        .ok()
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| RELEASES.to_string())
        .trim_end_matches('/')
        .to_string()
}

fn fetch(url: &str, dest: &Path) -> Result<(), String> {
    let mut curl = Command::new("curl");
    curl.args(["-fsSL", "--retry", "2", "--connect-timeout", "20"]);
    // Plain http only when $CPL_RELEASES_URL names an http mirror.
    if !url.starts_with("http://") {
        curl.args(["--proto", "=https,file", "--proto-redir", "=https"]);
    }
    let out = curl
        .arg("-o")
        .arg(dest)
        .arg(url)
        .output()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                "downloading toolchains needs curl on the PATH".to_string()
            }
            _ => format!("could not run curl: {e}"),
        })?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "could not download {url}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// Downloads the toolchain `selector` names, checks it against the
/// manifest's sha256 and unpacks it; nothing is installed on any failure.
fn install(home: &Path, selector: &Selector, log: &mut String) -> Result<String, String> {
    let base = releases_url();
    let dir_url = match selector.tag()? {
        Some(tag) => format!("{base}/download/{tag}"),
        None => format!("{base}/latest/download"),
    };
    let tmp = home.join("tmp");
    std::fs::create_dir_all(&tmp)
        .map_err(|e| format!("could not create {}: {e}", tmp.display()))?;
    let stamp = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    );
    let scratch = tmp.join(stamp);
    std::fs::create_dir_all(scratch.join("unpack"))
        .map_err(|e| format!("could not create {}: {e}", scratch.display()))?;
    let result = install_into(home, &dir_url, &scratch, selector, log);
    let _ = std::fs::remove_dir_all(&scratch);
    result
}

fn install_into(
    home: &Path,
    dir_url: &str,
    scratch: &Path,
    selector: &Selector,
    log: &mut String,
) -> Result<String, String> {
    let manifest = scratch.join("latest.json");
    fetch(&format!("{dir_url}/latest.json"), &manifest)?;
    let text = std::fs::read_to_string(&manifest)
        .map_err(|e| format!("could not read the manifest: {e}"))?;
    let release = parse_release(&text)?;
    if is_installed(home, &release.toolchain) {
        return Ok(release.toolchain);
    }
    let target = host_target().ok_or("there are no prebuilt toolchains for this platform")?;
    let (_, file, sha) = release
        .assets
        .iter()
        .find(|(t, _, _)| t == target)
        .ok_or_else(|| format!("{} has no build for {target}", release.toolchain))?;
    *log += &format!("cpl: installing {} ({selector})\n", release.toolchain);
    let archive = scratch.join(file);
    fetch(&format!("{dir_url}/{file}"), &archive)?;
    let bytes = std::fs::read(&archive).map_err(|e| format!("could not read {file}: {e}"))?;
    let actual = sha256::hex(&bytes);
    if &actual != sha {
        return Err(format!("{file} does not match its checksum (expected {sha}, got {actual}); nothing was installed"));
    }
    let unpack = scratch.join("unpack");
    let flag = if file.ends_with(".zip") {
        "-xf"
    } else {
        "-xzf"
    };
    let out = Command::new("tar")
        .arg(flag)
        .arg(&archive)
        .arg("-C")
        .arg(&unpack)
        .output()
        .map_err(|e| format!("could not run tar: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "could not unpack {file}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    if !binary(&unpack).is_file() {
        return Err(format!("{file} has no bin/cpl{EXE}"));
    }
    std::fs::write(unpack.join("release.json"), &text)
        .map_err(|e| format!("could not write release.json: {e}"))?;
    let dest = toolchain_dir(home, &release.toolchain);
    std::fs::create_dir_all(home.join("toolchains"))
        .map_err(|e| format!("could not create toolchains/: {e}"))?;
    if let Err(e) = std::fs::rename(&unpack, &dest) {
        // Another cpl may have installed the same toolchain meanwhile.
        if !is_installed(home, &release.toolchain) {
            return Err(format!("could not install into {}: {e}", dest.display()));
        }
    }
    Ok(release.toolchain)
}

/// The installed toolchain `selector` names, if any.
fn find_installed(home: &Path, settings: &Settings, selector: &Selector) -> Option<String> {
    match selector {
        Selector::Channel(channel) => settings
            .get(channel)
            .filter(|id| is_installed(home, id))
            .map(str::to_string),
        Selector::Version(id) | Selector::Toolchain(id) => {
            is_installed(home, id).then(|| id.clone())
        }
        Selector::Dated(tag) => installed_ids(home)
            .into_iter()
            .find(|id| release_tags(home, id).contains(tag)),
    }
}

/// The toolchain `selector` names, installed first if it isn't yet.
fn ensure(
    home: &Path,
    settings: &mut Settings,
    selector: &Selector,
    log: &mut String,
) -> Result<String, String> {
    if let Some(id) = find_installed(home, settings, selector) {
        return Ok(id);
    }
    let id = install(home, selector, log)?;
    if let Selector::Channel(channel) = selector {
        settings.set(channel, Some(&id));
        settings.save(home)?;
    }
    Ok(id)
}

/// The selector a command runs with, and why.
fn choose(
    plus: Option<&str>,
    cwd: &Path,
    settings: &Settings,
) -> Result<(Selector, String), String> {
    if let Some(text) = plus {
        return Ok((
            Selector::parse(text)?,
            format!("`+{text}` on the command line"),
        ));
    }
    if let Some(text) = std::env::var("CPL_VERSION")
        .ok()
        .filter(|v| !v.trim().is_empty())
    {
        let selector = Selector::parse(&text).map_err(|e| format!("{e} (in $CPL_VERSION)"))?;
        return Ok((selector, format!("CPL_VERSION={text}")));
    }
    for dir in cwd.ancestors() {
        let path = dir.join("cpl.toml");
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let pins: Vec<(String, String)> = top_level_keys(&text)
            .into_iter()
            .filter(|(k, _)| k == "version" || k == "channel")
            .collect();
        match pins.as_slice() {
            [] => continue,
            [(key, value)] => {
                let selector =
                    Selector::parse(value).map_err(|e| format!("{e} (in {})", path.display()))?;
                return Ok((
                    selector,
                    format!("{key} = \"{value}\" in {}", path.display()),
                ));
            }
            _ => {
                return Err(format!(
                    "{} sets both version and channel; keep one",
                    path.display()
                ))
            }
        }
    }
    let why = if settings.get("default").is_some() {
        "the default (cpl self channel)"
    } else {
        "the default channel"
    };
    Ok((settings.default_selector()?, why.to_string()))
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Runs as the launcher when this binary is `$CPL_HOME/bin/cpl`: picks the
/// toolchain, installs it on first use and hands the command to it. `None`
/// means this binary handles the command itself.
pub fn launch(args: &[String]) -> Option<u8> {
    let home = home()?;
    let launcher = home.join("bin").join(format!("cpl{EXE}"));
    let exe = std::env::current_exe().ok()?;
    let plus = args.first().and_then(|a| a.strip_prefix('+'));
    if !same_file(&exe, &launcher) {
        if plus.is_some() {
            eprintln!(
                "cpl: `+<toolchain>` works through the installed launcher, {}; install it from https://cp-lang.org/install.sh",
                launcher.display()
            );
            return Some(EXIT_USAGE);
        }
        return None;
    }
    let rest = if plus.is_some() { &args[1..] } else { args };
    if rest.first().map(String::as_str) == Some("self") {
        return None;
    }
    let mut settings = Settings::load(&home);
    let cwd = std::env::current_dir().unwrap_or_default();
    let mut log = String::new();
    let chosen = choose(plus, &cwd, &settings).and_then(|(selector, why)| {
        ensure(&home, &mut settings, &selector, &mut log).map(|id| (id, why))
    });
    eprint!("{log}");
    let (id, why) = match chosen {
        Ok(chosen) => chosen,
        Err(e) => {
            eprintln!("cpl: {e}");
            return Some(EXIT_ERRORS);
        }
    };
    let bin = binary(&toolchain_dir(&home, &id));
    if same_file(&bin, &exe) {
        return None;
    }
    let mut command = Command::new(&bin);
    command.args(rest).env("CPL_TOOLCHAIN", &id);
    if matches!(rest.first().map(String::as_str), Some("-V" | "--version")) {
        let status = command.status();
        let _ = std::io::stdout().flush();
        println!("toolchain {id}, chosen by {why}");
        return Some(
            status
                .ok()
                .and_then(|s| s.code())
                .map_or(EXIT_ERRORS, |c| c as u8),
        );
    }
    hand_over(command, &bin)
}

#[cfg(unix)]
fn hand_over(mut command: Command, bin: &Path) -> Option<u8> {
    use std::os::unix::process::CommandExt;
    let err = command.exec();
    eprintln!("cpl: could not start {}: {err}", bin.display());
    Some(EXIT_ERRORS)
}

#[cfg(not(unix))]
fn hand_over(mut command: Command, bin: &Path) -> Option<u8> {
    match command.status() {
        Ok(status) => Some(status.code().map_or(EXIT_ERRORS, |c| c as u8)),
        Err(err) => {
            eprintln!("cpl: could not start {}: {err}", bin.display());
            Some(EXIT_ERRORS)
        }
    }
}

/// `cpl self <command>`.
pub fn self_command(args: &[&str], stdout: &mut dyn Write, stderr: &mut String) -> u8 {
    let Some(home) = home() else {
        *stderr += "cpl: no home folder; set CPL_HOME\n";
        return EXIT_USAGE;
    };
    let mut settings = Settings::load(&home);
    let mut out = String::new();
    let result = match args {
        [] | ["-h" | "--help"] => {
            out += SELF_USAGE;
            Ok(())
        }
        ["update"] => update(&home, &mut settings, None, &mut out, stderr),
        ["update", channel] => update(&home, &mut settings, Some(channel), &mut out, stderr),
        ["channel"] => show_default(&home, &settings, &mut out),
        ["channel", toolchain] => set_default(&home, &mut settings, toolchain, &mut out, stderr),
        ["rollback"] => rollback(&home, &mut settings, &mut out),
        ["install", toolchain] => Selector::parse(toolchain)
            .and_then(|selector| ensure(&home, &mut settings, &selector, stderr))
            .map(|id| out += &format!("{id} is installed\n")),
        ["list"] => {
            list(&home, &settings, &mut out);
            Ok(())
        }
        _ => {
            *stderr += &format!("cpl: unknown `cpl self` command\n\n{SELF_USAGE}");
            return EXIT_USAGE;
        }
    };
    let _ = stdout.write_all(out.as_bytes());
    match result {
        Ok(()) => 0,
        Err(e) => {
            *stderr += &format!("cpl: {e}\n");
            EXIT_ERRORS
        }
    }
}

fn update(
    home: &Path,
    settings: &mut Settings,
    channel: Option<&str>,
    out: &mut String,
    log: &mut String,
) -> Result<(), String> {
    let default = settings.default_selector()?;
    let channel = match channel.map(Selector::parse).transpose()? {
        Some(Selector::Channel(channel)) => channel,
        Some(other) => return Err(format!("{other} is not a channel; update takes stable or edge")),
        None => match default {
            Selector::Channel(channel) => channel,
            other => {
                return Err(format!(
                    "the default is pinned to {other}; `cpl self channel stable` or `edge` follows a channel"
                ))
            }
        },
    };
    let current = find_installed(home, settings, &Selector::Channel(channel));
    let id = install(home, &Selector::Channel(channel), log)?;
    if current.as_deref() == Some(id.as_str()) {
        *out += &format!("{channel} is up to date: {id}\n");
        return Ok(());
    }
    settings.checkpoint(Some(channel));
    settings.set(channel, Some(&id));
    settings.save(home)?;
    if default == Selector::Channel(channel) {
        refresh_launcher(home, &id)?;
    }
    match current {
        Some(old) => {
            *out += &format!("{channel} updated: {old} -> {id} (`cpl self rollback` goes back)\n")
        }
        None => *out += &format!("{channel} installed: {id}\n"),
    }
    Ok(())
}

/// Replaces the launcher with `id`'s binary, when there is a launcher.
fn refresh_launcher(home: &Path, id: &str) -> Result<(), String> {
    let launcher = home.join("bin").join(format!("cpl{EXE}"));
    if !launcher.is_file() {
        return Ok(());
    }
    let fresh = home.join("bin").join(format!("cpl.new{EXE}"));
    let failed =
        |e: std::io::Error| format!("could not update the launcher {}: {e}", launcher.display());
    std::fs::copy(binary(&toolchain_dir(home, id)), &fresh).map_err(failed)?;
    if cfg!(windows) {
        // A running .exe can be renamed but not overwritten.
        let old = home.join("bin").join(format!("cpl.old{EXE}"));
        let _ = std::fs::remove_file(&old);
        std::fs::rename(&launcher, &old).map_err(failed)?;
    }
    std::fs::rename(&fresh, &launcher).map_err(failed)
}

fn show_default(home: &Path, settings: &Settings, out: &mut String) -> Result<(), String> {
    let default = settings.default_selector()?;
    match find_installed(home, settings, &default) {
        Some(id) => *out += &format!("{default}: {id}\n"),
        None => *out += &format!("{default}: not installed yet\n"),
    }
    Ok(())
}

fn set_default(
    home: &Path,
    settings: &mut Settings,
    text: &str,
    out: &mut String,
    log: &mut String,
) -> Result<(), String> {
    let selector = Selector::parse(text)?;
    let id = ensure(home, settings, &selector, log)?;
    settings.checkpoint(None);
    settings.set("default", Some(&selector.to_string()));
    settings.save(home)?;
    *out += &format!("default is now {selector}: {id}\n");
    Ok(())
}

fn rollback(home: &Path, settings: &mut Settings, out: &mut String) -> Result<(), String> {
    let Some(default) = settings.get("rollback-default").map(str::to_string) else {
        return Err("nothing to roll back to: no update or channel change yet".to_string());
    };
    let channel = settings.get("rollback-channel").map(str::to_string);
    let toolchain = settings.get("rollback-toolchain").map(str::to_string);
    if let Some(id) = toolchain.as_deref().filter(|id| !is_installed(home, id)) {
        return Err(format!("{id} is no longer installed"));
    }
    settings.checkpoint(channel.as_deref());
    settings.set("default", Some(&default));
    if let Some(channel) = &channel {
        settings.set(channel, toolchain.as_deref());
    }
    settings.save(home)?;
    let selector = settings.default_selector()?;
    let now =
        find_installed(home, settings, &selector).unwrap_or_else(|| "not installed".to_string());
    *out += &format!("rolled back: {selector} is {now} (`cpl self rollback` again undoes this)\n");
    Ok(())
}

fn list(home: &Path, settings: &Settings, out: &mut String) {
    let default = settings
        .default_selector()
        .ok()
        .and_then(|s| find_installed(home, settings, &s));
    let ids = installed_ids(home);
    if ids.is_empty() {
        *out += "no toolchains installed\n";
    }
    for id in ids {
        let mut marks: Vec<&str> = ["stable", "edge"]
            .into_iter()
            .filter(|c| settings.get(c) == Some(id.as_str()))
            .collect();
        if default.as_deref() == Some(id.as_str()) {
            marks.push("default");
        }
        if settings.get("rollback-toolchain") == Some(id.as_str()) {
            marks.push("rollback");
        }
        match marks.is_empty() {
            true => *out += &format!("{id}\n"),
            false => *out += &format!("{id} ({})\n", marks.join(", ")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selectors_name_channels_versions_and_dated_edge_builds() {
        assert_eq!(Selector::parse("stable"), Ok(Selector::Channel("stable")));
        assert_eq!(Selector::parse(" edge "), Ok(Selector::Channel("edge")));
        assert_eq!(
            Selector::parse("0.3.0"),
            Ok(Selector::Version("0.3.0".into()))
        );
        assert_eq!(
            Selector::parse("v0.3.0"),
            Ok(Selector::Version("0.3.0".into()))
        );
        assert_eq!(
            Selector::parse("edge-2026-11-22"),
            Ok(Selector::Dated("edge-2026-11-22".into()))
        );
        assert_eq!(
            Selector::parse("edge-2026-11-22-1a2b3c4d"),
            Ok(Selector::Toolchain("edge-2026-11-22-1a2b3c4d".into()))
        );
        for bad in [
            "nightly",
            "0.3",
            "edge-2026-11",
            "edge-2026-11-22-",
            "edge-2026-11-22-xyz",
            "1.2.3.4",
            "",
        ] {
            assert!(Selector::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn selectors_find_their_release() {
        let tag = |s: &str| Selector::parse(s).unwrap().tag();
        assert_eq!(tag("stable"), Ok(None));
        assert_eq!(tag("edge"), Ok(Some("edge".into())));
        assert_eq!(tag("0.3.0"), Ok(Some("v0.3.0".into())));
        assert_eq!(tag("edge-2026-11-22"), Ok(Some("edge-2026-11-22".into())));
        assert!(tag("edge-2026-11-22-1a2b3c4d")
            .unwrap_err()
            .contains("use edge or edge-2026-11-22"));
    }

    #[test]
    fn project_keys_stop_at_the_first_section() {
        let text = "# pins\nversion = \"0.3.0\"  # for the contest\nchannel=edge\n[libs]\nversion = \"9.9.9\"\n";
        assert_eq!(
            top_level_keys(text),
            vec![
                ("version".into(), "0.3.0".into()),
                ("channel".into(), "edge".into())
            ]
        );
    }

    #[test]
    fn manifests_reject_paths_outside_the_toolchain_folder() {
        let manifest = |id: &str, file: &str| {
            format!(
                r#"{{"toolchain": "{id}", "tags": ["edge"], "assets": {{"x": {{"file": "{file}", "sha256": "AB"}}}}}}"#
            )
        };
        let ok = parse_release(&manifest("edge-2026-11-22-1a2b3c4d", "cpl-x.tar.gz")).unwrap();
        assert_eq!(
            ok.assets,
            vec![("x".into(), "cpl-x.tar.gz".into(), "ab".into())]
        );
        assert_eq!(ok.tags, vec!["edge".to_string()]);
        assert!(parse_release(&manifest("../evil", "cpl-x.tar.gz")).is_err());
        assert!(parse_release(&manifest("0.3.0", "../cpl-x.tar.gz")).is_err());
        assert!(parse_release(&manifest("a/b", "cpl-x.tar.gz")).is_err());
    }

    #[test]
    fn every_release_target_maps_from_a_host() {
        assert!(host_target().is_some());
    }
}
