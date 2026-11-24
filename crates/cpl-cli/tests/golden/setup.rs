//! `cpl setup`: the editor files, extensions and plugins it installs.

use super::*;

/// `cpl setup neovim` writes the plugin, filetype, syntax and queries where
/// Neovim loads a start package; `vim` leaves out what only Neovim reads.
#[test]
fn setup_writes_the_editor_files() {
    let dir = scratch("setup");
    let out = cpl(&dir, &["setup", "neovim", "--dir", "nvim"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).starts_with("installed CP Lang for neovim in nvim"),
        "{}",
        text(&out.stdout)
    );
    for file in [
        "plugin/cplang.lua",
        "ftdetect/cp.vim",
        "syntax/cp.vim",
        "queries/cp/highlights.scm",
    ] {
        assert!(dir.join("nvim").join(file).is_file(), "{file}");
    }
    let out = cpl(&dir, &["setup", "vim", "--dir", "vim"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        dir.join("vim/plugin/cplang.vim").is_file() && !dir.join("vim/plugin/cplang.lua").exists()
    );

    let out = cpl(&dir, &["setup", "nvim", "--dir", "nvim", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let json = cpl_json::parse(&text(&out.stdout)).expect("JSON");
    assert_eq!(json.at(&["editor"]).as_str(), Some("neovim"));
    assert_eq!(json.at(&["ok"]).as_bool(), Some(true));
    assert!(json.at(&["message"]).as_str().unwrap().contains("in nvim"));
    // A folder that is a file: the same failure, as JSON on stdout.
    std::fs::write(dir.join("taken"), "").unwrap();
    let out = cpl(&dir, &["setup", "vim", "--dir", "taken", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out.stdout));
    let json = cpl_json::parse(&text(&out.stdout)).expect("JSON");
    assert_eq!(json.at(&["ok"]).as_bool(), Some(false));
    assert!(json
        .at(&["error"])
        .as_str()
        .unwrap()
        .starts_with("can't write taken"));

    let out = cpl(&dir, &["setup", "emacs"]);
    assert_eq!(out.status.code(), Some(EXIT_USAGE as i32));
    assert!(
        text(&out.stderr).contains("no setup for `emacs`"),
        "{}",
        text(&out.stderr)
    );
}

/// `cpl setup vscode --vsix FILE` hands the file to `code --install-extension`.
#[cfg(unix)]
#[test]
fn setup_installs_the_vscode_extension() {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch("setup-vscode");
    let code = dir.join("code");
    std::fs::write(
        &code,
        "#!/bin/sh\nif [ \"$1\" = --list-extensions ]; then echo cp-lang.cp-lang@0.3.0; exit; fi\necho \"$@\" > \"$(dirname \"$0\")/args\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&code, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(dir.join("x.vsix"), "").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["setup", "vscode", "--vsix", "x.vsix"])
        .current_dir(&dir)
        .env("CPL_CODE", &code)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        std::fs::read_to_string(dir.join("args")).unwrap(),
        "--install-extension x.vsix --force\n"
    );
    assert!(
        text(&out.stdout).contains("cp-lang.cp-lang@0.3.0"),
        "{}",
        text(&out.stdout)
    );
    // An editor command that installs nothing is a failure, not a success.
    let silent = dir.join("silent");
    std::fs::write(&silent, "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&silent, std::fs::Permissions::from_mode(0o755)).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["setup", "vscode", "--vsix", "x.vsix"])
        .current_dir(&dir)
        .env("CPL_CODE", &silent)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("does not list cp-lang.cp-lang"),
        "{}",
        text(&out.stderr)
    );
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["setup", "vscode", "--vsix", "x.vsix"])
        .current_dir(&dir)
        .env("CPL_CODE", dir.join("nope"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("set CPL_CODE"),
        "{}",
        text(&out.stderr)
    );
}

/// `cpl setup jetbrains --zip FILE` unpacks the plugin into each 2024.3 or
/// newer IDE found under the config folder, checks a Community IDE has
/// LSP4IJ, and `--check` reads each IDE's log for whether it loaded.
#[cfg(target_os = "linux")]
#[test]
fn setup_unpacks_the_jetbrains_plugin_into_each_ide() {
    let dir = scratch("setup-jetbrains");
    for ide in [
        "IntelliJIdea2025.1",
        "IdeaIC2024.3",
        "IntelliJIdea2024.2",
        "consentOptions",
    ] {
        std::fs::create_dir_all(dir.join("config/JetBrains").join(ide)).unwrap();
    }
    // Every IDE gets LSP4IJ. The version setup pins is present already, so
    // the test downloads nothing.
    let pinned = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../editors/jetbrains/gradle.properties"),
    )
    .unwrap()
    .lines()
    .find_map(|line| {
        line.strip_prefix("lsp4ijVersion = ")
            .map(|v| v.trim().to_string())
    })
    .unwrap();
    for ide in ["IntelliJIdea2025.1", "IdeaIC2024.3"] {
        let lib = dir.join("data/JetBrains").join(ide).join("lsp4ij/lib");
        std::fs::create_dir_all(&lib).unwrap();
        std::fs::write(lib.join(format!("lsp4ij-{pinned}.jar")), "x").unwrap();
    }
    let made = Command::new("python3")
        .args([
            "-c",
            "import zipfile; zipfile.ZipFile('p.zip', 'w').writestr('cp-lang-jetbrains/lib/x.jar', 'x')",
        ])
        .current_dir(&dir)
        .status();
    if !made.map(|s| s.success()).unwrap_or(false) {
        return;
    }
    let setup = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_cpl"))
            .arg("setup")
            .args(args)
            .current_dir(&dir)
            .env("HOME", &dir)
            .env("XDG_CONFIG_HOME", dir.join("config"))
            .env("XDG_DATA_HOME", dir.join("data"))
            .env("XDG_CACHE_HOME", dir.join("cache"))
            .output()
            .unwrap()
    };
    let out = setup(&["jetbrains", "--zip", "p.zip"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    for ide in ["IntelliJIdea2025.1", "IdeaIC2024.3"] {
        assert!(dir
            .join("data/JetBrains")
            .join(ide)
            .join("cp-lang-jetbrains/lib/x.jar")
            .is_file());
    }
    assert!(!dir.join("data/JetBrains/IntelliJIdea2024.2").exists());
    assert!(!dir.join("data/JetBrains/consentOptions").exists());
    // The IDE learns where the plugin's updates come from.
    let updates =
        std::fs::read_to_string(dir.join("config/JetBrains/IdeaIC2024.3/options/updates.xml"))
            .unwrap();
    assert!(
        updates.contains("/download/edge/updatePlugins.xml"),
        "{updates}"
    );
    let said = text(&out.stdout);
    assert!(
        said.contains("IdeaIC2024.3") && said.contains("with LSP4IJ"),
        "{said}"
    );
    assert!(said.contains("skipped IntelliJIdea2024.2"), "{said}");

    // No IDE has started since: --check fails, naming each.
    let out = setup(&["jetbrains", "--check"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stderr).contains("IdeaIC2024.3: no log yet"),
        "{}",
        text(&out.stderr)
    );
    let log = |ide: &str, body: &str| {
        let path = dir.join("cache/JetBrains").join(ide).join("log");
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("idea.log"), body).unwrap();
    };
    let started = "2026-11-23 INFO - #c.i.i.i.AppStarter - ------ IDE STARTED ------\n";
    log(
        "IntelliJIdea2025.1",
        &format!("{started}INFO - Loaded custom plugins: CP Lang (0.3.0)\n"),
    );
    log("IdeaIC2024.3", &format!("{started}WARN - Plugin 'CP Lang' requires plugin 'com.intellij.modules.ultimate' to be installed\n"));
    let out = setup(&["jetbrains", "--check"]);
    assert_eq!(out.status.code(), Some(1));
    let said = text(&out.stderr);
    assert!(
        said.contains("IntelliJIdea2025.1: CP Lang (0.3.0) loaded without LSP4IJ"),
        "{said}"
    );
    assert!(
        said.contains("IdeaIC2024.3: CP Lang did not load") && said.contains("modules.ultimate"),
        "{said}"
    );
    for ide in ["IntelliJIdea2025.1", "IdeaIC2024.3"] {
        log(
            ide,
            &format!("{started}INFO - Loaded custom plugins: CP Lang (0.3.0), LSP4IJ (0.19.4)\n"),
        );
    }
    let out = setup(&["jetbrains", "--check"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).contains("IntelliJIdea2025.1: CP Lang loaded"),
        "{}",
        text(&out.stdout)
    );
}

/// An IDE that has never started has no config folder yet, but its install's
/// `product-info.json` names the folder it will use, and `cpl setup` finds it
/// through its launcher on the PATH.
#[cfg(target_os = "linux")]
#[test]
fn setup_finds_a_jetbrains_ide_that_has_never_started() {
    let dir = scratch("setup-jetbrains-fresh");
    let install = dir.join("idea-IU-253.28294.334");
    std::fs::create_dir_all(install.join("bin")).unwrap();
    std::fs::write(
        install.join("product-info.json"),
        r#"{"name": "IntelliJ IDEA", "version": "2025.3", "dataDirectoryName": "IntelliJIdea2025.3"}"#,
    )
    .unwrap();
    std::fs::write(install.join("bin/idea.sh"), "#!/bin/sh\n").unwrap();
    let pinned = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../editors/jetbrains/gradle.properties"),
    )
    .unwrap()
    .lines()
    .find_map(|line| {
        line.strip_prefix("lsp4ijVersion = ")
            .map(|v| v.trim().to_string())
    })
    .unwrap();
    let lib = dir.join("data/JetBrains/IntelliJIdea2025.3/lsp4ij/lib");
    std::fs::create_dir_all(&lib).unwrap();
    std::fs::write(lib.join(format!("lsp4ij-{pinned}.jar")), "x").unwrap();
    let made = Command::new("python3")
        .args([
            "-c",
            "import zipfile; zipfile.ZipFile('p.zip', 'w').writestr('cp-lang-jetbrains/lib/x.jar', 'x')",
        ])
        .current_dir(&dir)
        .status();
    if !made.map(|s| s.success()).unwrap_or(false) {
        return;
    }
    let path = std::env::join_paths(std::iter::once(install.join("bin")).chain(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
    ))
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["setup", "jetbrains", "--zip", "p.zip"])
        .current_dir(&dir)
        .env("HOME", &dir)
        .env("PATH", path)
        .env("XDG_CONFIG_HOME", dir.join("config"))
        .env("XDG_DATA_HOME", dir.join("data"))
        .env("XDG_CACHE_HOME", dir.join("cache"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(dir
        .join("data/JetBrains/IntelliJIdea2025.3/cp-lang-jetbrains/lib/x.jar")
        .is_file());
    assert!(
        text(&out.stdout).contains("IntelliJIdea2025.3"),
        "{}",
        text(&out.stdout)
    );
}

/// The one step on a fresh machine: an empty home where the installer put
/// `cpl` at `~/.cpl/bin` and no `cpl` is on the PATH. `cpl setup neovim`,
/// `vim` and `emacs`, with no `--dir`, write where each editor loads a
/// package, with the path of the `cpl` setup ran as in its settings
/// (`g:cpl_path`, which Neovim's `vim.lsp.start` and vim-lsp both run, and
/// `cp-cpl-program`, eglot's), and that `cpl` answers as `cpl lsp`. Vim's
/// folder is whichever the Vim on the PATH reads (on Windows, `vimfiles` for
/// native Vim, `.vim` for Git Bash's). Vim already has yegappan/lsp, so setup
/// downloads nothing (smoke_vim.sh covers the vim-lsp download).
#[test]
fn setup_on_a_fresh_home_installs_where_the_editor_finds_cpl_lsp() {
    let dir = scratch("setup-fresh-home");
    let home = dir.join("home");
    let exe = if cfg!(windows) { "cpl.exe" } else { "cpl" };
    let installed = home.join(".cpl/bin").join(exe);
    std::fs::create_dir_all(installed.parent().unwrap()).unwrap();
    std::fs::copy(env!("CARGO_BIN_EXE_cpl"), &installed).unwrap();
    // The launcher hands each command to the toolchain install.sh unpacked.
    let toolchain = home.join(".cpl/toolchains/edge-local/bin").join(exe);
    std::fs::create_dir_all(toolchain.parent().unwrap()).unwrap();
    std::fs::copy(env!("CARGO_BIN_EXE_cpl"), &toolchain).unwrap();
    std::fs::write(home.join(".cpl/settings.toml"), "stable = \"edge-local\"\n").unwrap();
    let nvim_site = if cfg!(windows) {
        home.join("AppData/Local/nvim-data/site")
    } else {
        home.join(".local/share/nvim/site")
    };
    for vim_dir in [".vim", "vimfiles"] {
        let client = home
            .join(vim_dir)
            .join("pack/plugins/start/lsp/autoload/lsp/lsp.vim");
        std::fs::create_dir_all(client.parent().unwrap()).unwrap();
        std::fs::write(&client, "").unwrap();
    }
    let path = std::env::join_paths(
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
            .filter(|d| !d.join(exe).exists()),
    )
    .unwrap();
    let fresh = |program: &Path, args: &[&str]| {
        let mut command = Command::new(program);
        command.args(args).current_dir(&dir).env("PATH", &path);
        command.env("HOME", &home).env("USERPROFILE", &home);
        command.env("LOCALAPPDATA", home.join("AppData/Local"));
        command.env("CPL_RELEASES_URL", "file:///no-network");
        for var in ["XDG_DATA_HOME", "XDG_CONFIG_HOME", "CPL_HOME", "CPL_STD"] {
            command.env_remove(var);
        }
        command.output().unwrap()
    };
    let runtime = [
        "plugin/cplang.vim",
        "plugin/cpl_path.vim",
        "ftdetect/cp.vim",
        "syntax/cp.vim",
    ];
    let mut packages = vec![];
    for editor in ["neovim", "vim"] {
        let out = fresh(&installed, &["setup", editor]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{editor}: {}",
            text(&out.stderr)
        );
        let candidates = match editor {
            "neovim" => vec![nvim_site.clone()],
            _ => vec![home.join(".vim"), home.join("vimfiles")],
        };
        let written: Vec<_> = candidates
            .iter()
            .map(|d| d.join("pack/cpl/start/cplang"))
            .filter(|p| runtime.iter().all(|f| p.join(f).is_file()))
            .collect();
        assert_eq!(written.len(), 1, "{editor}: {}", text(&out.stdout));
        packages.push((editor, written[0].clone()));
    }
    assert!(packages[0].1.join("plugin/cplang.lua").is_file());
    assert!(text(&fresh(&installed, &["setup", "vim"]).stdout)
        .contains("yegappan/lsp already installed"));

    // Vim opens a .cp file from each package, and the client's `cpl lsp`
    // answers `initialize`.
    std::fs::write(dir.join("a.cp"), "print(1)\n").unwrap();
    std::fs::write(
        dir.join("probe.vim"),
        r#"let s:said = ['{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}',
    \ '{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}',
    \ '{"jsonrpc":"2.0","method":"exit","params":null}']
let s:input = join(map(s:said, {_, m -> 'Content-Length: ' . len(m) . "\r\n\r\n" . m}), '')
let s:out = system(shellescape(g:cpl_path) . ' lsp', s:input)
call writefile([&filetype, g:cpl_path, s:out =~# '"capabilities"' ? 'answers' : s:out], 'found')
"#,
    )
    .unwrap();
    for (editor, package) in &packages {
        let _ = std::fs::remove_file(dir.join("found"));
        let rtp = format!(
            "set rtp^={}",
            package.display().to_string().replace('\\', "/")
        );
        let pre: &[&str] = if *editor == "vim" {
            &["-Nu", "NORC"]
        } else {
            &[
                "-Nu",
                "NONE",
                "-c",
                &rtp,
                "-c",
                "runtime! plugin/cpl_path.vim",
            ]
        };
        let args = [
            pre,
            &[
                "-i",
                "NONE",
                "-Es",
                "-c",
                "filetype plugin on",
                "-c",
                "edit a.cp",
            ],
            &["-S", "probe.vim", "-c", "qall!"],
        ]
        .concat();
        let out = fresh(Path::new("vim"), &args);
        let found = std::fs::read_to_string(dir.join("found")).unwrap_or_default();
        let found: Vec<_> = found.lines().map(|l| l.replace('\\', "/")).collect();
        assert!(
            found.len() == 3
                && found[0] == "cp"
                && found[1] == installed.display().to_string().replace('\\', "/")
                && found[2] == "answers",
            "{editor}: {found:?} {}",
            text(&out.stderr)
        );
    }

    // Emacs (`just install-emacs`): `package-activate-all`, which its startup
    // runs, loads the package, and a .cp file's eglot command answers.
    {
        let out = fresh(&installed, &["setup", "emacs"]);
        assert_eq!(out.status.code(), Some(0), "emacs: {}", text(&out.stderr));
        let said = [
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}"#,
            r#"{"jsonrpc":"2.0","method":"exit","params":null}"#,
        ];
        let input: String = said
            .iter()
            .map(|m| format!("Content-Length: {}\r\n\r\n{m}", m.len()))
            .collect();
        std::fs::write(dir.join("lsp.in"), input).unwrap();
        let _ = std::fs::remove_file(dir.join("found"));
        let probe = r#"(progn
  (package-activate-all)
  (find-file "a.cp")
  (let ((mode major-mode) (command (cp--server-command)))
    (with-temp-file "found"
      (insert (format "%s\n%s\n" mode (car command)))
      (insert (if (with-temp-buffer
                    (apply #'call-process (car command) "lsp.in" t nil (cdr command))
                    (goto-char (point-min))
                    (search-forward "\"capabilities\"" nil t))
                  "answers" "silent")))))"#;
        let out = fresh(Path::new("emacs"), &["--batch", "--eval", probe]);
        let found = std::fs::read_to_string(dir.join("found")).unwrap_or_default();
        let found: Vec<_> = found.lines().map(|l| l.replace('\\', "/")).collect();
        assert!(
            found.len() == 3
                && found[0] == "cp-mode"
                && found[1] == installed.display().to_string().replace('\\', "/")
                && found[2] == "answers",
            "emacs: {found:?} {}",
            text(&out.stderr)
        );
    }
}
