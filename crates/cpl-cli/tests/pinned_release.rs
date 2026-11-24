//! A program pinned to an old release rebuilds byte for byte with every
//! later `cpl` (R11.4.2): `pinned_release/cpl.toml` pins a dated edge
//! release, this build of `cpl` runs as the launcher in a fresh `$CPL_HOME`
//! and installs that toolchain from the published release on first use, and
//! every frozen v0.2 port (`tests/v02-snapshot/`) built in that folder must
//! give what `pinned_release/<version>.txt` records: its exit code and the
//! sha256 of its C++ and of its diagnostics. `BLESS=1` rewrites the record
//! from the pinned release.

use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "support/parallel.rs"]
mod parallel;
#[path = "../src/sha256.rs"]
mod sha256;

const CPL: &str = env!("CARGO_BIN_EXE_cpl");

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/pinned_release")
}

fn pinned_version() -> String {
    let toml = std::fs::read_to_string(project().join("cpl.toml")).unwrap();
    let line = toml
        .lines()
        .find_map(|l| l.trim().strip_prefix("version = "))
        .expect("pinned_release/cpl.toml pins a `version`");
    line.trim_matches('"').to_string()
}

/// The launcher, a fresh copy of this build, and the `$CPL_HOME` it is in.
fn launcher() -> (PathBuf, PathBuf) {
    let home = Path::new(env!("CARGO_TARGET_TMPDIR")).join("pinned-release-home");
    let bin = home
        .join("bin")
        .join(format!("cpl{}", std::env::consts::EXE_SUFFIX));
    std::fs::create_dir_all(bin.parent().unwrap()).unwrap();
    std::fs::copy(CPL, &bin).unwrap();
    (bin, home)
}

fn command(bin: &Path, home: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(bin);
    command
        .args(args)
        .current_dir(project())
        .env("CPL_HOME", home)
        .env_remove("CPL_VERSION")
        .env_remove("CPL_TOOLCHAIN")
        .env_remove("CPL_RELEASES_URL");
    command
}

/// The frozen ports, as paths relative to the pinned project, so the
/// diagnostics name the same file on every machine.
fn ports() -> Vec<(String, String)> {
    let mut ids: Vec<String> = std::fs::read_dir(root().join("tests/v02-snapshot"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join("port.cp").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    ids.sort();
    ids.into_iter()
        .map(|id| {
            let path = format!("../../../../tests/v02-snapshot/{id}/port.cp");
            (id, path)
        })
        .collect()
}

#[test]
fn a_port_pinned_to_an_old_release_builds_the_same_cpp() {
    let version = pinned_version();
    let (bin, home) = launcher();
    let picked = command(&bin, &home, &["--version"]).output().unwrap();
    let said = String::from_utf8_lossy(&picked.stdout);
    assert!(
        picked.status.success()
            && said.contains(&format!("toolchain {version}"))
            && said.contains("cpl.toml"),
        "the launcher should pick `{version}` from pinned_release/cpl.toml:\n{said}{}",
        String::from_utf8_lossy(&picked.stderr)
    );
    let ports = ports();
    // A build per port, so every core runs one.
    let got: Vec<String> = parallel::parallel(&ports, |(id, path)| {
        let out = command(&bin, &home, &["build", path, "-o", "-"])
            .output()
            .unwrap();
        let code = out.status.code().unwrap_or(-1);
        let cpp = sha256::hex(&out.stdout);
        let diags = sha256::hex(&out.stderr);
        format!("{id} {code} {cpp} {diags}")
    });
    let record = project().join(format!("{version}.txt"));
    let got = got.join("\n") + "\n";
    if std::env::var_os("BLESS").is_some() {
        std::fs::write(&record, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&record).unwrap_or_default();
    let want: Vec<&str> = want.lines().collect();
    let differ: Vec<&str> = got
        .lines()
        .filter(|line| !want.contains(line))
        .chain(
            want.iter()
                .copied()
                .filter(|line| !got.lines().any(|g| g == *line)),
        )
        .collect();
    let mut ids: Vec<&str> = differ.iter().filter_map(|l| l.split(' ').next()).collect();
    ids.dedup();
    assert!(
        differ.is_empty(),
        "{} of {} frozen ports no longer build byte for byte with `{version}` (id, exit code, \
         sha256 of the C++ and of the diagnostics; new and recorded lines):\n{}\n\
         A new frozen copy is recorded with BLESS=1 cargo test -p cpl-cli --test pinned_release",
        ids.len(),
        ports.len(),
        differ.join("\n")
    );
}
