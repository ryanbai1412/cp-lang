use super::*;

fn dir(name: &str, toml: Option<&str>) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("cpl-judge-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    if let Some(toml) = toml {
        std::fs::write(dir.join("cp.toml"), toml).unwrap();
    }
    dir
}

#[test]
fn every_builtin_has_a_distinct_name_a_compiler_and_a_limit() {
    let profiles = builtins();
    assert_eq!(profiles[0].name, "generic");
    for want in [
        "generic",
        "codeforces",
        "atcoder",
        "cses",
        "icpc",
        "kattis",
        "dmoj",
        "usaco",
    ] {
        assert!(builtin(want).is_some(), "{want}");
    }
    let mut names: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
    names.dedup();
    assert_eq!(names.len(), profiles.len());
    for p in &profiles {
        assert!(
            !p.compiler.is_empty() && p.source_limit >= 64 * 1024,
            "{}",
            p.name
        );
        assert!(p.warn_at() < p.source_limit);
    }
    assert_eq!(builtin("generic").unwrap().warn_at(), 24 * 1024);
}

#[test]
fn every_builtin_aims_under_32_kb_warns_past_24_kb_and_stops_past_64_kb() {
    for p in builtins() {
        assert_eq!(
            (p.warn_at(), p.aim(), p.limit()),
            (24 * 1024, 32 * 1024, 64 * 1024),
            "{}",
            p.name
        );
    }
    let toml = "[judge.tiny]\nextends = \"generic\"\nsource_kb = 40\n";
    let tiny = resolve(
        &dir("tiny-limit", Some(toml)).join("a.cp"),
        "// judge: tiny\n",
        None,
    )
    .unwrap();
    assert_eq!(
        (tiny.warn_at(), tiny.aim(), tiny.limit()),
        (24 * 1024, 32 * 1024, 40 * 1024)
    );
}

#[test]
fn the_contest_judges_have_their_compiler_stack_and_usual_limits() {
    let want = [
        (
            "codeforces",
            "-std=c++20",
            Stack::Bytes(256 << 20),
            2000,
            256,
        ),
        ("atcoder", "-std=gnu++23", Stack::Unknown, 2000, 1024),
        ("dmoj", "-std=c++17", Stack::Unlimited, 2000, 256),
        ("cses", "-std=c++17", Stack::Unknown, 1000, 512),
        ("usaco", "-std=c++17", Stack::Unlimited, 2000, 256),
    ];
    for (name, std, stack, time_ms, memory_mb) in want {
        let p = builtin(name).unwrap();
        assert!(
            p.flags.iter().any(|f| f == std) && p.flags.iter().any(|f| f == "-O2"),
            "{name}: {:?}",
            p.flags
        );
        assert_eq!(
            (p.stack, p.time_ms, p.memory_mb),
            (stack, time_ms, memory_mb),
            "{name}"
        );
    }
    let toml = "[judge.cses]\ntime_ms = 3000\nmemory_mb = 64\n";
    let p = resolve(
        &dir("judge-limits", Some(toml)).join("a.cp"),
        "// judge: cses\n",
        None,
    )
    .unwrap();
    assert_eq!((p.time_ms, p.memory_mb), (3000, 64));
}

#[test]
fn every_builtin_runs_the_program_on_a_1_gb_thread_stack() {
    for profile in builtins() {
        assert_eq!(profile.thread_stack, Some(1 << 30), "{}", profile.name);
        assert_eq!(profile.thread_warning(), None, "{}", profile.name);
    }
}

#[test]
fn a_project_sets_the_thread_stack_or_says_the_judge_allows_no_threads() {
    let toml = "[judge]\njudge = \"olympiad\"\n[judge.olympiad]\nextends = \"generic\"\nthread_stack_mb = 0\n";
    let banned = dir("threads", Some(toml));
    let profile = resolve(&banned.join("a.cp"), "", None).unwrap();
    assert_eq!(profile.thread_stack, None);
    let warning = profile.thread_warning().unwrap();
    assert!(
        warning.contains("judge `olympiad` allows no threads") && warning.contains("8 MB stack"),
        "{warning}"
    );
    let toml = "[judge.generic]\nthread_stack_mb = 512\n";
    let other = dir("thread-stack", Some(toml));
    assert_eq!(
        resolve(&other.join("a.cp"), "", None).unwrap().thread_stack,
        Some(512 << 20)
    );
}

#[test]
fn the_flag_beats_the_file_which_beats_the_project() {
    let dir = dir("order", Some("[judge]\njudge = \"cses\"\n"));
    let file = dir.join("sub/a.cp");
    assert_eq!(resolve(&file, "", None).unwrap().name, "cses");
    assert_eq!(
        resolve(&file, "// judge: atcoder\n", None).unwrap().name,
        "atcoder"
    );
    assert_eq!(
        resolve(&file, "// judge: atcoder\n", Some("icpc"))
            .unwrap()
            .name,
        "icpc"
    );
    let alone = std::env::temp_dir().join("cpl-judge-no-project-here/a.cp");
    assert_eq!(resolve(&alone, "", None).unwrap().name, "generic");
}

#[test]
fn a_project_changes_a_profile_or_adds_one() {
    let toml = "\
[limits]
time_ms = 1000

[judge]
judge = \"school\"  # the course's own judge

[judge.school]
extends = \"icpc\"
title = \"School judge\"
stack_mb = 64
source_kb = 100
flags = [\"-std=c++17\", \"-O2\"]

[judge.cses]
int128 = false
";
    let dir = dir("custom", Some(toml));
    let file = dir.join("sub/a.cp");
    let school = resolve(&file, "", None).unwrap();
    assert_eq!(school.name, "school");
    assert_eq!(school.compiler, "g++");
    assert_eq!(school.flags, ["-std=c++17", "-O2"]);
    assert_eq!(school.stack, Stack::Bytes(64 << 20));
    assert_eq!(school.source_limit, 100 * 1024);
    let cses = resolve(&file, "", Some("cses")).unwrap();
    assert!(!cses.int128);
    assert!(cses
        .unsupported("__int128 x;")
        .unwrap()
        .contains("judge `cses`"));
    assert!(builtin("cses")
        .unwrap()
        .unsupported("__int128 x;")
        .is_none());
}

#[test]
fn a_wrong_name_or_key_says_what_is_allowed() {
    let dir = dir("wrong", None);
    let file = dir.join("a.cp");
    let error = resolve(&file, "// judge: topcoder\n", None).unwrap_err();
    assert!(
        error.contains("`// judge:` line names judge `topcoder`"),
        "{error}"
    );
    assert!(error.contains("codeforces, atcoder"), "{error}");
    let error = Config::parse("[judge.cses]\nstack = 5\n").unwrap_err();
    assert!(
        error.contains("line 2: a judge has no `stack`; its keys are extends,"),
        "{error}"
    );
    let error = Config::parse("[judge.mine]\nstack_mb = 5\n").unwrap_err();
    assert!(error.contains("says which one it `extends`"), "{error}");
    let error = Config::parse("[judge.cses]\nstack_mb = \"lots\"\n").unwrap_err();
    assert!(
        error.contains("`stack_mb` is a whole number of MB, or \"unlimited\""),
        "{error}"
    );
    let error = Config::parse("[judge]\nname = \"cses\"\n").unwrap_err();
    assert!(
        error.contains("`[judge]` has `judge = \"<name>\"`"),
        "{error}"
    );
    std::fs::write(dir.join("cp.toml"), "[judge]\njudge = \"nope\"\n").unwrap();
    let error = resolve(&file, "", None).unwrap_err();
    assert!(error.contains("cp.toml` names judge `nope`"), "{error}");
}

#[test]
fn a_profile_that_extends_itself_in_a_loop_is_not_found() {
    let config = Config::parse("[judge.a]\nextends = \"b\"\n[judge.b]\nextends = \"a\"\n").unwrap();
    assert!(config.profile("a").is_none());
}

#[test]
fn the_stamp_names_the_judge_its_command_and_its_stack() {
    let stamp = builtin("icpc").unwrap().stamp();
    assert_eq!(
        stamp,
        "judge `icpc`: g++ -x c++ -Wall -O2 -static -pipe; unlimited stack"
    );
    assert!(builtin("cses")
        .unwrap()
        .stamp()
        .ends_with("stack not documented (8 MB assumed)"));
}

#[test]
fn an_older_compiler_gets_the_older_spelling_of_the_standard() {
    assert_eq!(
        older_spelling("-std=gnu++23").as_deref(),
        Some("-std=gnu++2b")
    );
    assert_eq!(older_spelling("-std=c++20").as_deref(), Some("-std=c++2a"));
    assert_eq!(older_spelling("-O2"), None);
}

#[test]
fn the_local_build_keeps_what_the_compiler_here_accepts() {
    if !accepts("g++", None) {
        eprintln!("no g++ here");
        return;
    }
    let mut profile = builtin("generic").unwrap();
    profile.flags.push("-fno-such-flag-anywhere".into());
    let local = profile.local();
    assert_eq!(local.flags, ["-std=c++17", "-O2"]);
    assert_eq!(
        local.dropped,
        [("-fno-such-flag-anywhere".to_string(), None)]
    );
}

#[test]
fn a_flag_and_its_value_or_a_whole_archive_span_are_kept_together() {
    let flags: Vec<String> = "-x c++ -O2 -Wl,--whole-archive -lpthread -Wl,--no-whole-archive -s"
        .split(' ')
        .map(String::from)
        .collect();
    let groups: Vec<String> = groups(&flags).iter().map(|g| g.join(" ")).collect();
    assert_eq!(
        groups,
        [
            "-x c++",
            "-O2",
            "-Wl,--whole-archive -lpthread -Wl,--no-whole-archive",
            "-s"
        ]
    );
}

#[test]
#[cfg(unix)] // the fake compiler is a shell script
fn a_yes_from_the_probe_is_kept_across_processes_and_a_no_is_not() {
    use std::os::unix::fs::PermissionsExt;
    let root = dir("probe-memo", None);
    let (memo, log) = (root.join("memo"), root.join("log"));
    let compiler = root.join("cxx");
    let write = |exit: u8| {
        let script = format!("#!/bin/sh\necho run >> '{}'\nexit {exit}\n", log.display());
        std::fs::write(&compiler, script).unwrap();
        std::fs::set_permissions(&compiler, std::fs::Permissions::from_mode(0o755)).unwrap();
    };
    let runs = || std::fs::read_to_string(&log).map_or(0, |s| s.lines().count());
    let cxx = compiler.to_str().unwrap();
    let flags = vec!["-O2".to_string()];
    write(1);
    assert!(!probe(cxx, &flags, Some(&memo)));
    assert!(!probe(cxx, &flags, Some(&memo)));
    assert_eq!(runs(), 2);
    write(0);
    assert!(probe(cxx, &flags, Some(&memo)));
    assert!(probe(cxx, &flags, Some(&memo)));
    assert_eq!(runs(), 3);
    assert!(probe(cxx, &["-O0".to_string()], Some(&memo)));
    assert_eq!(runs(), 4);
    std::fs::write(
        &compiler,
        "#!/bin/sh\necho replaced >> \"$0.log\"\nexit 1\n",
    )
    .unwrap();
    assert!(!probe(cxx, &flags, Some(&memo)));
    assert!(!probe(cxx, &flags, None));
}
