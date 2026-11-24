//! `cpl/command` and `cpl/commands`: how an editor runs `cpl` for a lens or
//! a typed command, and which command names it completes.

use super::*;

fn ask(server: &mut Server, method: &str, params: Value) -> Value {
    server.handle(&request(1, "initialize", obj([])));
    let reply = server.handle(&request(2, method, params));
    reply[0].at(&["result"]).clone()
}

fn strings(list: &Value) -> Vec<&str> {
    list.as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect()
}

#[test]
fn a_command_runs_on_the_file_and_stress_and_locate_report_a_view() {
    let run = |params: Value| {
        let result = ask(&mut Server::new(), "cpl/command", params);
        let argv = strings(result.at(&["argv"])).join(" ");
        (argv, result.at(&["view"]) == &Value::Bool(true))
    };
    let typed = |args: &[&str]| {
        let args: Vec<Value> = args.iter().map(|a| Value::from(*a)).collect();
        obj([("arguments", args.into()), ("file", "/p/a.cp".into())])
    };
    assert_eq!(run(typed(&[])), ("test /p/a.cp".into(), false));
    assert_eq!(run(obj([("arguments", obj([]))])), ("test".into(), false));
    assert_eq!(run(typed(&["check"])), ("check /p/a.cp".into(), false));
    assert_eq!(
        run(typed(&["stress", "--trials", "9"])),
        ("stress /p/a.cp --trials 9 --json".into(), true)
    );
    assert_eq!(
        run(typed(&["locate", "i1"])),
        ("locate /p/a.cp i1 --json".into(), true)
    );
    // A lens's arguments already name the file.
    let lens = obj([(
        "arguments",
        vec![Value::from("run"), "/p/a.cp".into()].into(),
    )]);
    assert_eq!(run(lens), ("run /p/a.cp".into(), false));
    let lens = obj([(
        "arguments",
        vec![Value::from("stress"), "/p/a.cp".into()].into(),
    )]);
    assert_eq!(run(lens), ("stress /p/a.cp --json".into(), true));
}

#[test]
fn command_names_complete_only_the_first_word() {
    let names = vec!["submit", "stress", "test"];
    let complete = |line: &str| {
        let mut server = Server::new().with_commands(names.clone());
        let result = ask(&mut server, "cpl/commands", obj([("line", line.into())]));
        strings(&result)
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
    };
    assert_eq!(complete("Cpl s"), ["submit", "stress"]);
    assert_eq!(complete("Cpl su"), ["submit"]);
    assert_eq!(complete("Cpl "), ["submit", "stress", "test"]);
    assert!(complete("Cpl stress ").is_empty());
    assert!(complete("Cpl stress --tr").is_empty());
    assert!(complete("Cpl x").is_empty());
}
