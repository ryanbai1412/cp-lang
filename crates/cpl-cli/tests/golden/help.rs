//! Every command's `--help`, `help --json`, JSON flags and usage errors.

use super::*;

/// Every command `cpl --help` lists: its own `--help`, and an option it does
/// not have is a usage error that shows its usage, not a missing file.
/// Enforces `handbook/guidelines/commands-checklist.md#r1-help`.
#[test]
fn every_command_has_help_and_rejects_an_unknown_option() {
    let help = text(&cpl(Path::new("."), &["--help"]).stdout);
    let commands: Vec<&str> = help
        .lines()
        .skip_while(|line| *line != "commands:")
        .skip(1)
        .take_while(|line| line.starts_with("  "))
        .filter(|line| !line.starts_with("   "))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    assert_eq!(commands.len(), 28, "{help}");
    for command in commands {
        helps(command);
        let out = cpl(Path::new("."), &[command, "--no-such-option"]);
        let stderr = text(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(EXIT_USAGE.into()),
            "{command}: {stderr}"
        );
        let wording = format!(
            "`{command}` has no option `--no-such-option`; `cpl {command} --help` lists them"
        );
        assert!(stderr.contains(&wording), "{command}: {stderr}");
        assert!(shows_usage(&stderr, command), "{command}: {stderr}");
        assert!(!stderr.contains("can't read"), "{command}: {stderr}");
    }
}

#[test]
fn help_json_lists_every_command_with_its_usage_summary_json_and_help() {
    let out = cpl(Path::new("."), &["--help", "--json"]);
    assert_eq!(out.status.code(), Some(0));
    let json = cpl_json::parse(&text(&out.stdout)).expect("cpl --help --json is JSON");
    let commands = json.at(&["commands"]).as_array().unwrap();
    assert_eq!(commands.len(), 28);
    let listed = text(&cpl(Path::new("."), &["--help"]).stdout);
    let no_json: Vec<_> = commands
        .iter()
        .filter(|c| c.at(&["json"]).as_str().is_none())
        .map(|c| c.at(&["name"]).as_str().unwrap())
        .collect();
    assert_eq!(no_json, ["lsp", "dap"]);
    for command in commands {
        let name = command.at(&["name"]).as_str().unwrap();
        let usage = command.at(&["usage"]).as_str().unwrap();
        assert!(listed.contains(&format!("\n  {usage}")), "{name}: {usage}");
        assert!(
            !command.at(&["summary"]).as_str().unwrap().is_empty(),
            "{name}"
        );
        assert!(
            shows_usage(command.at(&["help"]).as_str().unwrap_or(""), name),
            "{name}"
        );
    }
}

#[test]
fn check_json_reports_each_file_and_its_diagnostics() {
    let dir = scratch("check-json");
    std::fs::write(
        dir.join("ok.cp"),
        "fn inc(x: Int) -> Int:\n    return x + 1\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("bad.cp"),
        "fn inc(x: Int) -> Int:\n    return y + 1\n",
    )
    .unwrap();
    let out = cpl(&dir, &["check", "ok.cp", "bad.cp", "--json"]);
    assert_eq!(
        out.status.code(),
        Some(EXIT_ERRORS.into()),
        "{}",
        text(&out.stderr)
    );
    assert!(out.stderr.is_empty(), "{}", text(&out.stderr));
    let json = cpl_json::parse(&text(&out.stdout)).expect("cpl check --json is JSON");
    let files = json.at(&["files"]).as_array().unwrap();
    assert_eq!(files[0].at(&["ok"]).as_bool(), Some(true));
    assert_eq!(
        files[0].at(&["diagnostics"]).as_array().map(<[_]>::len),
        Some(0)
    );
    assert_eq!(files[1].at(&["path"]).as_str(), Some("bad.cp"));
    assert_eq!(files[1].at(&["ok"]).as_bool(), Some(false));
    let first = &files[1].at(&["diagnostics"]).as_array().unwrap()[0];
    assert!(
        cpl_diag::lookup(first.at(&["code"]).as_str().unwrap()).is_some(),
        "{first:?}"
    );
    assert_eq!(first.at(&["severity"]).as_str(), Some("error"));
    assert_eq!(first.at(&["start", "line"]).as_i64(), Some(2));
    assert_eq!(first.at(&["start", "col"]).as_i64(), Some(12));
    // With no file named, the newest `.cp` file in the folder is checked.
    let newest = cpl(&dir, &["check", "--json"]);
    let json = cpl_json::parse(&text(&newest.stdout)).expect("JSON");
    assert_eq!(json.at(&["files"]).as_array().unwrap().len(), 1);
}

/// Enforces `handbook/guidelines/commands-checklist.md#r3-one-spelling`.
#[test]
fn json_and_format_json_are_one_flag_on_every_command_with_json() {
    let dir = scratch("json-spelling");
    std::fs::write(
        dir.join("inc.cp"),
        "/// Adds one.\nfn inc(x: Int) -> Int:\n    return x + 1\n",
    )
    .unwrap();
    for (json, format) in [
        (
            &["check", "inc.cp", "--json"][..],
            &["check", "inc.cp", "--format", "json"][..],
        ),
        (&["judges", "--json"], &["judges", "--format", "json"]),
        (
            &["fix", "inc.cp", "--json"][..],
            &["fix", "inc.cp", "--format", "json"][..],
        ),
        (
            &["doc", "--json", "inc.cp"],
            &["doc", "--format", "json", "inc.cp"],
        ),
    ] {
        let (json, format) = (cpl(&dir, json), cpl(&dir, format));
        assert_eq!(json.status.code(), Some(0), "{}", text(&json.stderr));
        assert!(
            cpl_json::parse(&text(&json.stdout)).is_ok(),
            "{}",
            text(&json.stdout)
        );
        assert_eq!(text(&json.stdout), text(&format.stdout));
    }
}

/// Help fits an 80-column terminal. `stress` is the debugger domain's, which
/// has the request (plan/domains/debugger/worklog.md).
/// Enforces `handbook/guidelines/commands-checklist.md#r1-help`.
#[test]
fn every_help_fits_80_columns() {
    let json = cpl_json::parse(&text(&cpl(Path::new("."), &["--help", "--json"]).stdout)).unwrap();
    let mut helps = vec![(
        "cpl".to_string(),
        text(&cpl(Path::new("."), &["--help"]).stdout),
    )];
    for command in json.at(&["commands"]).as_array().unwrap() {
        let name = command.at(&["name"]).as_str().unwrap();
        if name != "stress" {
            helps.push((
                name.into(),
                command.at(&["help"]).as_str().unwrap_or("").into(),
            ));
        }
    }
    let wide: Vec<String> = helps
        .iter()
        .flat_map(|(name, help)| help.lines().map(move |line| (name, line)))
        .filter(|(_, line)| line.chars().count() > 80)
        .map(|(name, line)| format!("{name}: {line}"))
        .collect();
    assert!(wide.is_empty(), "{wide:#?}");
}

/// A usage error in a contest command speaks as `cpl` and names an
/// invocation that works.
#[test]
fn usage_errors_name_a_working_invocation() {
    let dir = scratch("usage-errors");
    for args in [
        &["run"][..],
        &["test"],
        &["new"],
        &["stress"],
        &["build"],
        &["setup"],
        &["fetch"],
        &["submit"],
        &["listen"],
        &["tset"],
    ] {
        let out = cpl(&dir, args);
        let err = text(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(EXIT_USAGE.into()),
            "{args:?}: {err}"
        );
        let first = err.lines().next().unwrap_or("");
        assert!(
            first.starts_with("cpl: ") && first.contains("`cpl "),
            "{args:?}: {first}"
        );
    }
}
