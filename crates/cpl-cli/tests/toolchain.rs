//! The launcher and `cpl self` against fake releases on disk (file://
//! URLs), packed by scripts/release/package.py and described by
//! scripts/release/manifest.py, the same scripts the release workflow runs.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const CPL: &str = env!("CARGO_BIN_EXE_cpl");

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn host_target() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "aarch64") => "aarch64-unknown-linux-musl",
        ("linux", _) => "x86_64-unknown-linux-musl",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", _) => "x86_64-apple-darwin",
        _ => "x86_64-pc-windows-msvc",
    }
}

struct World {
    dir: PathBuf,
    releases: PathBuf,
    home: PathBuf,
}

impl World {
    fn new(name: &str) -> World {
        let dir = std::env::temp_dir().join(format!("cpl-toolchain-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let releases = dir.join("releases");
        let home = dir.join("home");
        std::fs::create_dir_all(home.join("bin")).unwrap();
        std::fs::create_dir_all(dir.join("project/src")).unwrap();
        World { dir, releases, home }
    }

    /// Publishes a release under `releases/download/<tag>/` for each tag;
    /// `stable` also becomes the newest release (`releases/latest/download`).
    fn publish(&self, toolchain: &str, channel: &str, tags: &[&str]) {
        let staging = self.dir.join("staging").join(toolchain);
        let _ = std::fs::remove_dir_all(&staging);
        let python = |args: &[&str]| {
            let out = Command::new("python3").args(args).current_dir(root()).output().expect("python3 runs");
            assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        };
        python(&["scripts/release/package.py", host_target(), CPL, staging.to_str().unwrap()]);
        let mut manifest = vec![
            "scripts/release/manifest.py",
            staging.to_str().unwrap(),
            "--toolchain",
            toolchain,
            "--version",
            "0.3.0",
            "--channel",
            channel,
            "--commit",
            "0123456789abcdef",
        ];
        for tag in tags {
            manifest.extend(["--tag", tag]);
        }
        python(&manifest);
        let mut folders: Vec<PathBuf> = tags.iter().map(|t| self.releases.join("download").join(t)).collect();
        if channel == "stable" {
            folders.push(self.releases.join("latest/download"));
        }
        for folder in folders {
            let _ = std::fs::remove_dir_all(&folder);
            std::fs::create_dir_all(&folder).unwrap();
            for entry in std::fs::read_dir(&staging).unwrap() {
                let entry = entry.unwrap();
                std::fs::copy(entry.path(), folder.join(entry.file_name())).unwrap();
            }
        }
    }

    fn url(&self) -> String {
        let path = self.releases.to_str().unwrap().replace('\\', "/");
        format!("file://{}{path}", if path.starts_with('/') { "" } else { "/" })
    }

    fn launcher(&self) -> PathBuf {
        self.home.join("bin").join(format!("cpl{}", std::env::consts::EXE_SUFFIX))
    }

    fn cpl_in(&self, exe: &Path, cwd: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut command = Command::new(exe);
        command
            .args(args)
            .current_dir(cwd)
            .env("CPL_HOME", &self.home)
            .env("CPL_RELEASES_URL", self.url())
            .env_remove("CPL_VERSION")
            .env_remove("CPL_STD");
        for (key, value) in env {
            command.env(key, value);
        }
        command.output().expect("cpl runs")
    }

    fn cpl(&self, args: &[&str], env: &[(&str, &str)]) -> Output {
        self.cpl_in(&self.launcher(), &self.dir.join("project/src"), args, env)
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn ok(out: &Output) -> String {
    assert!(
        out.status.success(),
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn err(out: &Output) -> String {
    assert!(!out.status.success(), "expected a failure, got:\n{}", String::from_utf8_lossy(&out.stdout));
    String::from_utf8(out.stderr.clone()).unwrap()
}

fn installed(world: &World) -> Vec<String> {
    let mut ids: Vec<String> = std::fs::read_dir(world.home.join("toolchains"))
        .map(|dir| dir.flatten().map(|e| e.file_name().into_string().unwrap()).collect())
        .unwrap_or_default();
    ids.sort();
    ids
}

#[test]
fn the_launcher_picks_installs_and_runs_toolchains_in_precedence_order() {
    let world = World::new("select");
    world.publish("0.3.0", "stable", &["v0.3.0"]);
    world.publish("edge-2026-11-21-aaaaaaaa", "edge", &["edge", "edge-2026-11-21"]);
    std::fs::copy(CPL, world.launcher()).unwrap();

    let out = world.cpl(&["--version"], &[]);
    let stdout = ok(&out);
    assert!(stdout.starts_with("cpl "), "{stdout}");
    assert!(stdout.ends_with("toolchain 0.3.0, chosen by the default channel\n"), "{stdout}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("cpl: installing 0.3.0 (stable)"));

    let stdout = ok(&world.cpl(&["+edge", "--version"], &[]));
    assert!(stdout.ends_with("toolchain edge-2026-11-21-aaaaaaaa, chosen by `+edge` on the command line\n"), "{stdout}");
    assert_eq!(installed(&world), ["0.3.0", "edge-2026-11-21-aaaaaaaa"]);

    std::fs::write(world.dir.join("project/cpl.toml"), "version = \"0.3.0\"\n").unwrap();
    let pin = format!("version = \"0.3.0\" in {}", world.dir.join("project").join("cpl.toml").display());
    let stdout = ok(&world.cpl(&["--version"], &[]));
    assert!(stdout.ends_with(&format!("toolchain 0.3.0, chosen by {pin}\n")), "{stdout}");

    std::fs::write(world.dir.join("project/cpl.toml"), "channel = \"edge\"\n").unwrap();
    let stdout = ok(&world.cpl(&["--version"], &[]));
    assert!(stdout.contains("toolchain edge-2026-11-21-aaaaaaaa, chosen by channel = \"edge\" in "), "{stdout}");

    let stdout = ok(&world.cpl(&["--version"], &[("CPL_VERSION", "0.3.0")]));
    assert!(stdout.ends_with("toolchain 0.3.0, chosen by CPL_VERSION=0.3.0\n"), "{stdout}");

    let offline = [("CPL_VERSION", "0.3.0"), ("CPL_RELEASES_URL", "file:///cpl-offline-mirror-is-absent")];
    let stdout = ok(&world.cpl(&["+edge-2026-11-21", "--version"], &offline));
    assert!(stdout.contains("toolchain edge-2026-11-21-aaaaaaaa, chosen by `+edge-2026-11-21`"), "{stdout}");

    let program = world.dir.join("project/src/hello.cp");
    std::fs::write(&program, "print(\"hi\")\n").unwrap();
    assert_eq!(ok(&world.cpl(&["+0.3.0", "run", "hello.cp"], &offline)), "hi\n");

    assert!(err(&world.cpl(&["+nightly", "run", "hello.cp"], &[])).contains("`nightly` is not a toolchain"));
    std::fs::write(world.dir.join("project/cpl.toml"), "version = \"0.3.0\"\nchannel = \"edge\"\n").unwrap();
    assert!(err(&world.cpl(&["--version"], &[])).contains("sets both version and channel"));

    let direct = world.cpl_in(Path::new(CPL), &world.dir, &["+edge", "--version"], &[]);
    assert!(err(&direct).contains("works through the installed launcher"));
}

#[test]
fn self_update_switches_channels_and_rolls_back_offline() {
    let world = World::new("self");
    world.publish("0.3.0", "stable", &["v0.3.0"]);
    world.publish("edge-2026-11-21-aaaaaaaa", "edge", &["edge", "edge-2026-11-21"]);
    std::fs::copy(CPL, world.launcher()).unwrap();

    assert_eq!(ok(&world.cpl(&["self", "list"], &[])), "no toolchains installed\n");
    assert_eq!(ok(&world.cpl(&["self", "update"], &[])), "stable installed: 0.3.0\n");
    assert_eq!(ok(&world.cpl(&["self", "update"], &[])), "stable is up to date: 0.3.0\n");
    assert_eq!(ok(&world.cpl(&["self", "channel", "edge"], &[])), "default is now edge: edge-2026-11-21-aaaaaaaa\n");
    let stdout = ok(&world.cpl(&["--version"], &[]));
    assert!(stdout.ends_with("toolchain edge-2026-11-21-aaaaaaaa, chosen by the default (cpl self channel)\n"), "{stdout}");

    world.publish("edge-2026-11-22-bbbbbbbb", "edge", &["edge", "edge-2026-11-22"]);
    assert_eq!(
        ok(&world.cpl(&["self", "update"], &[])),
        "edge updated: edge-2026-11-21-aaaaaaaa -> edge-2026-11-22-bbbbbbbb (`cpl self rollback` goes back)\n"
    );
    assert_eq!(
        ok(&world.cpl(&["self", "list"], &[])),
        "0.3.0 (stable)\nedge-2026-11-21-aaaaaaaa (rollback)\nedge-2026-11-22-bbbbbbbb (edge, default)\n"
    );

    let offline = [("CPL_RELEASES_URL", "file:///cpl-offline-mirror-is-absent")];
    assert_eq!(
        ok(&world.cpl(&["self", "rollback"], &offline)),
        "rolled back: edge is edge-2026-11-21-aaaaaaaa (`cpl self rollback` again undoes this)\n"
    );
    let stdout = ok(&world.cpl(&["--version"], &offline));
    assert!(stdout.contains("toolchain edge-2026-11-21-aaaaaaaa"), "{stdout}");
    assert_eq!(
        ok(&world.cpl(&["self", "rollback"], &offline)),
        "rolled back: edge is edge-2026-11-22-bbbbbbbb (`cpl self rollback` again undoes this)\n"
    );
    ok(&world.cpl(&["self", "channel", "stable"], &offline));
    assert_eq!(ok(&world.cpl(&["self", "channel"], &offline)), "stable: 0.3.0\n");
    assert_eq!(
        ok(&world.cpl(&["self", "rollback"], &offline)),
        "rolled back: edge is edge-2026-11-22-bbbbbbbb (`cpl self rollback` again undoes this)\n"
    );
    assert!(err(&world.cpl(&["self", "update"], &offline)).contains("could not download"));
}

#[test]
fn a_download_that_fails_its_checksum_installs_nothing() {
    let world = World::new("checksum");
    world.publish("0.4.0", "stable", &["v0.4.0"]);
    for folder in ["latest/download", "download/v0.4.0"] {
        let archive = std::fs::read_dir(world.releases.join(folder))
            .unwrap()
            .flatten()
            .find(|e| e.file_name().to_string_lossy().starts_with("cpl-"))
            .unwrap()
            .path();
        let mut bytes = std::fs::read(&archive).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        std::fs::write(&archive, bytes).unwrap();
    }
    let stderr = err(&world.cpl_in(Path::new(CPL), &world.dir, &["self", "install", "0.4.0"], &[]));
    assert!(stderr.contains("does not match its checksum"), "{stderr}");
    assert!(installed(&world).is_empty(), "{:?}", installed(&world));
    let stderr = err(&world.cpl_in(Path::new(CPL), &world.dir, &["self", "install", "edge-2026-11-21-aaaaaaaa"], &[]));
    assert!(stderr.contains("single edge builds can't be downloaded"), "{stderr}");
}

fn install_sh(world: &World, args: &[&str]) -> Output {
    let shell = if Path::new("/usr/bin/dash").exists() { "/usr/bin/dash" } else { "sh" };
    Command::new(shell)
        .arg(root().join("web/public/install.sh"))
        .args(args)
        .current_dir(&world.dir)
        .env("HOME", &world.dir)
        .env("CPL_HOME", &world.home)
        .env("CPL_RELEASES_URL", world.url())
        .env("PATH", std::env::var("PATH").unwrap())
        .output()
        .expect("sh runs")
}

#[cfg(unix)]
#[test]
fn install_sh_installs_the_launcher_and_a_checked_toolchain() {
    let world = World::new("install-sh");
    world.publish("0.3.0", "stable", &["v0.3.0"]);
    world.publish("edge-2026-11-21-aaaaaaaa", "edge", &["edge", "edge-2026-11-21"]);
    std::fs::remove_dir(world.home.join("bin")).unwrap();

    let stdout = ok(&install_sh(&world, &[]));
    assert!(stdout.contains("cpl-install: installed 0.3.0 into "), "{stdout}");
    let path_line = format!("export PATH=\"{}:$PATH\"", world.home.join("bin").display());
    assert!(stdout.contains(&path_line), "{stdout}");
    let profile = std::fs::read_to_string(world.dir.join(".profile")).unwrap();
    assert_eq!(profile.matches(&path_line).count(), 1, "{profile}");
    let stdout = ok(&world.cpl(&["--version"], &[("CPL_RELEASES_URL", "file:///cpl-offline-mirror-is-absent")]));
    assert!(stdout.ends_with("toolchain 0.3.0, chosen by the default channel\n"), "{stdout}");

    let stdout = ok(&install_sh(&world, &["--edge", "--no-modify-path"]));
    assert!(stdout.contains("installed edge-2026-11-21-aaaaaaaa"), "{stdout}");
    let stdout = ok(&world.cpl(&["--version"], &[]));
    assert!(stdout.ends_with("toolchain edge-2026-11-21-aaaaaaaa, chosen by the default (cpl self channel)\n"), "{stdout}");
    assert_eq!(ok(&world.cpl(&["self", "list"], &[])), "0.3.0 (stable)\nedge-2026-11-21-aaaaaaaa (edge, default)\n");

    let stdout = ok(&install_sh(&world, &["--version", "0.3.0"]));
    assert!(stdout.contains("0.3.0 is already installed"), "{stdout}");
    assert_eq!(ok(&world.cpl(&["self", "channel"], &[])), "0.3.0: 0.3.0\n");
    let profile = std::fs::read_to_string(world.dir.join(".profile")).unwrap();
    assert_eq!(profile.matches(&path_line).count(), 1, "{profile}");

    world.publish("edge-2026-11-22-bbbbbbbb", "edge", &["edge", "edge-2026-11-22"]);
    let archive = world.releases.join("download/edge").join(format!("cpl-{}.tar.gz", host_target()));
    std::fs::write(&archive, b"not the archive").unwrap();
    let stderr = err(&install_sh(&world, &["--edge"]));
    assert!(stderr.contains("does not match its checksum"), "{stderr}");
    assert_eq!(installed(&world), ["0.3.0", "edge-2026-11-21-aaaaaaaa"]);

    assert!(err(&install_sh(&world, &["--version", "nightly"])).contains("--version takes"));
}

fn install_ps1(world: &World, args: &[&str]) -> Option<Output> {
    let pwsh = std::env::var_os("PATH")
        .and_then(|path| std::env::split_paths(&path).map(|dir| dir.join("pwsh")).find(|p| p.is_file()))?;
    let script = root().join("web/public/install.ps1");
    Some(
        Command::new(pwsh)
            .args(["-NoProfile", "-NonInteractive", "-File"])
            .arg(script)
            .args(args)
            .current_dir(&world.dir)
            .env("HOME", &world.dir)
            .env("CPL_HOME", &world.home)
            .env("CPL_RELEASES_URL", world.url())
            .env("CPL_INSTALL_TARGET", "x86_64-pc-windows-msvc")
            .output()
            .expect("pwsh runs"),
    )
}

/// install.ps1 under PowerShell 7 when it is installed, against Windows
/// archives; the launcher itself is covered above.
#[cfg(unix)]
#[test]
fn install_ps1_installs_a_checked_windows_toolchain() {
    let world = World::new("install-ps1");
    std::fs::remove_dir(world.home.join("bin")).unwrap();
    let Some(out) = install_ps1(&world, &["-Edge"]) else { return };
    assert!(err(&out).contains("could not download"), "{}", String::from_utf8_lossy(&out.stdout));
    let windows = |toolchain: &str, tags: &[&str]| {
        let staging = world.dir.join("staging-win");
        let _ = std::fs::remove_dir_all(&staging);
        let run = |args: &[&str]| assert!(Command::new("python3").args(args).current_dir(root()).status().unwrap().success());
        run(&["scripts/release/package.py", "x86_64-pc-windows-msvc", CPL, staging.to_str().unwrap()]);
        let mut manifest = vec!["scripts/release/manifest.py", staging.to_str().unwrap(), "--toolchain", toolchain];
        manifest.extend(["--version", "0.3.0", "--channel", "edge", "--commit", "0123abcd"]);
        for tag in tags {
            manifest.extend(["--tag", tag]);
        }
        run(&manifest);
        for tag in tags {
            let folder = world.releases.join("download").join(tag);
            let _ = std::fs::remove_dir_all(&folder);
            std::fs::create_dir_all(&folder).unwrap();
            for entry in std::fs::read_dir(&staging).unwrap().flatten() {
                std::fs::copy(entry.path(), folder.join(entry.file_name())).unwrap();
            }
        }
    };
    windows("edge-2026-11-21-aaaaaaaa", &["edge", "edge-2026-11-21"]);

    let stdout = ok(&install_ps1(&world, &["-Edge"]).unwrap());
    assert!(stdout.contains("installed edge-2026-11-21-aaaaaaaa"), "{stdout}");
    assert!(stdout.contains("to your PATH"), "{stdout}");
    let toolchain = world.home.join("toolchains/edge-2026-11-21-aaaaaaaa");
    assert_eq!(std::fs::read(toolchain.join("bin/cpl.exe")).unwrap(), std::fs::read(CPL).unwrap());
    assert!(toolchain.join("stdlib/prelude.cp").is_file());
    assert!(toolchain.join("release.json").is_file());
    assert_eq!(std::fs::read(world.home.join("bin/cpl.exe")).unwrap(), std::fs::read(CPL).unwrap());
    let settings = std::fs::read_to_string(world.home.join("settings.toml")).unwrap();
    assert_eq!(settings, "default = \"edge\"\nedge = \"edge-2026-11-21-aaaaaaaa\"\n");

    let stdout = ok(&install_ps1(&world, &["-Version", "edge-2026-11-21"]).unwrap());
    assert!(stdout.contains("already installed"), "{stdout}");
    let settings = std::fs::read_to_string(world.home.join("settings.toml")).unwrap();
    assert_eq!(settings, "edge = \"edge-2026-11-21-aaaaaaaa\"\ndefault = \"edge-2026-11-21\"\n");

    windows("edge-2026-11-22-bbbbbbbb", &["edge"]);
    let archive = world.releases.join("download/edge/cpl-x86_64-pc-windows-msvc.zip");
    std::fs::write(&archive, b"not the archive").unwrap();
    assert!(err(&install_ps1(&world, &["-Edge"]).unwrap()).contains("does not match its checksum"));
    assert_eq!(installed(&world), ["edge-2026-11-21-aaaaaaaa"]);
}
