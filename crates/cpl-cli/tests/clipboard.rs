//! `cpl b -c` reaches each OS's clipboard tool: pbcopy (macOS), wl-copy,
//! xclip and xsel (Linux), clip.exe and clip (Windows and WSL). Each is
//! stubbed alone on PATH; the stub must get the built C++ byte for byte, with
//! the arguments that select the clipboard.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("clipboard")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::write(dir.join("a.cp"), "let n: Int = read()\nprint(n * 2)\n").unwrap();
    dir
}

fn stub(dir: &Path, tool: &str) {
    let path = dir.join("bin").join(tool);
    std::fs::write(
        &path,
        "#!/bin/sh\necho \"$*\" > \"$CLIP_ARGS\"\n/bin/cat > \"$CLIP_OUT\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

/// `cpl b a.cp -c` with only the stubs on PATH: (exit, stderr).
fn build_and_copy(dir: &Path) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["b", "a.cp", "-c"])
        .current_dir(dir)
        .env_remove("CPL_CLIPBOARD")
        .env("PATH", dir.join("bin"))
        .env("CLIP_OUT", dir.join("clip"))
        .env("CLIP_ARGS", dir.join("args"))
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn each_os_clipboard_tool_gets_the_built_cpp() {
    let tools = [
        ("pbcopy", ""),
        ("wl-copy", ""),
        ("xclip", "-selection clipboard"),
        ("xsel", "--clipboard --input"),
        ("clip.exe", ""),
        ("clip", ""),
    ];
    for (tool, args) in tools {
        let dir = scratch(tool);
        stub(&dir, tool);
        let (code, stderr) = build_and_copy(&dir);
        assert_eq!(code, 0, "{tool}: {stderr}");
        let named = format!("{tool} {args}");
        assert!(
            stderr.contains(&format!("with `{}`", named.trim())),
            "{tool}: {stderr}"
        );
        let cpp = std::fs::read(dir.join("a.cpp")).expect("cpl b writes a.cpp");
        assert!(cpp.len() > 100, "{tool}: a.cpp is {} bytes", cpp.len());
        assert_eq!(std::fs::read(dir.join("clip")).unwrap(), cpp, "{tool}");
        let got = std::fs::read_to_string(dir.join("args")).unwrap();
        assert_eq!(got.trim(), args, "{tool}");
    }
}

#[test]
fn the_first_tool_wins_and_no_tool_warns() {
    let dir = scratch("both");
    stub(&dir, "pbcopy");
    stub(&dir, "xclip");
    let (code, stderr) = build_and_copy(&dir);
    assert_eq!(code, 0, "{stderr}");
    assert!(stderr.contains("with `pbcopy`"), "{stderr}");

    let dir = scratch("none");
    let (code, stderr) = build_and_copy(&dir);
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stderr.contains("no clipboard tool here") && stderr.contains("CPL_CLIPBOARD"),
        "{stderr}"
    );
    assert!(dir.join("a.cpp").is_file(), "the C++ is still in the file");
    assert!(!dir.join("clip").exists());
}
