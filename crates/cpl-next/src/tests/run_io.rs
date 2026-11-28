//! Running on streamed input and output (R10.4).

use super::*;

/// Input streamed as the program reads gives what the same bytes given whole give, on the
/// reference interpreter and the engine.
#[test]
fn streamed_input_reads_as_fixed_input_does() {
    use cpl_next_engine::{Io, Limits, RunInput};
    struct Trickle(Vec<u8>, usize);
    impl std::io::Read for Trickle {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let Some(&b) = self.0.get(self.1) else {
                return Ok(0);
            };
            buf[0] = b;
            self.1 += 1;
            Ok(1)
        }
    }
    let src = "let n: 1..=100 = read()\nlet xs: [n]Int = read()\nprint(n, xs)\n";
    let text = b"3\n1 2  3\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let (p, fs) = c.form(Form::Raw);
    let fixed = || RunInput {
        io: Io::Fixed(text.to_vec()),
        limits: Limits::default(),
        out: None,
    };
    let streamed = || RunInput {
        io: Io::Stream(Box::new(Trickle(text.to_vec(), 0))),
        limits: Limits::default(),
        out: None,
    };
    let want = cpl_next_interp::run(p, fs, fixed());
    assert!(
        want.trap.is_none() && !want.output.is_empty(),
        "{}",
        want.output
    );
    let runs = [
        cpl_next_interp::run(p, fs, streamed()),
        cpl_next_engine::run(&c.prog, &c.opt, streamed()),
        cpl_next_engine::run(&c.prog, &c.opt, fixed()),
    ];
    for o in runs {
        assert!(o.trap.is_none());
        assert_eq!(o.output, want.output);
    }
}

/// A judge that answers each query only once the run has handed it over: each `(query, reply)`
/// of its script sends `reply` when what it has seen so far ends with `query`.
#[derive(Clone)]
struct Peer {
    seen: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    script: std::sync::Arc<std::sync::Mutex<Vec<(&'static str, &'static str)>>>,
    asked: std::sync::Arc<std::sync::Mutex<usize>>,
}
impl Peer {
    fn new(script: &[(&'static str, &'static str)]) -> Peer {
        Peer {
            seen: Default::default(),
            script: std::sync::Arc::new(std::sync::Mutex::new(
                script.iter().rev().copied().collect(),
            )),
            asked: Default::default(),
        }
    }
    fn seen(&self) -> String {
        String::from_utf8_lossy(&self.seen.lock().expect("seen")).into_owned()
    }
}
impl std::io::Write for Peer {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.seen.lock().expect("seen").extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl std::io::Read for Peer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        *self.asked.lock().expect("asked") += 1;
        let Some((query, reply)) = self.script.lock().expect("script").pop() else {
            return Ok(0);
        };
        let seen = self.seen();
        assert!(
            seen.ends_with(query),
            "waited for an answer to {query:?} before handing it over: {seen:?}"
        );
        buf[..reply.len()].copy_from_slice(reply.as_bytes());
        Ok(reply.len())
    }
}

/// A read that must wait for more input hands the judge the output first, in every mode
/// (R10.4): `wire.ask`'s query and a plain `print` before a `read` alike, while a read the
/// buffer already holds waits for nothing; the rest goes at exit. The same on the reference
/// interpreter, the engine and the C++.
#[test]
fn a_read_that_waits_hands_the_judge_the_output_first() {
    let src = "let wire = Wire()\nlet a: Int = wire.ask(\"? 1\")\nlet b: Int = read()\n\
print(\"? {a + b}\")\nlet c: Int = read()\nprint(\"! {c}\")\n";
    let script = [("? 1\n", "5\n7\n"), ("? 12\n", "3\n")];
    let want = "? 1\n? 12\n! 3\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let (p, fs) = c.form(Form::Raw);
    for engine in [false, true] {
        let peer = Peer::new(&script);
        let input = c.streamed(Box::new(peer.clone()), Box::new(peer.clone()));
        let o = match engine {
            true => c.run_on(input),
            false => cpl_next_interp::run(p, fs, input),
        };
        assert!(o.trap.is_none(), "{engine}");
        assert_eq!(
            (peer.seen().as_str(), o.output.as_str()),
            (want, want),
            "{engine}"
        );
        assert_eq!(
            *peer.asked.lock().expect("asked"),
            2,
            "{engine}: one wait per query"
        );
    }
    let (bin, _) = cxx::build(
        c.cpp().as_bytes(),
        &["-std=c++17", "-O1", "-w", "-x", "c++"],
    )
    .expect("g++ builds it");
    let mut ch = std::process::Command::new(bin)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("runs");
    let mut out = ch.stdout.take().expect("stdout");
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut b = [0; 256];
        while let Ok(n @ 1..) = std::io::Read::read(&mut out, &mut b) {
            drop(tx.send(b[..n].to_vec()));
        }
    });
    let mut seen = Vec::new();
    let mut to = ch.stdin.take().expect("stdin");
    for (query, reply) in script {
        while !seen.ends_with(query.as_bytes()) {
            let got = rx.recv_timeout(std::time::Duration::from_secs(20));
            seen.extend(got.unwrap_or_else(|_| panic!("C++ never handed over {query:?}")));
        }
        std::io::Write::write_all(&mut to, reply.as_bytes()).expect("answers");
    }
    drop(to);
    seen.extend(rx.iter().flatten());
    assert!(ch.wait().expect("ends").success());
    assert_eq!(String::from_utf8_lossy(&seen), want);
}

/// The last token reads the same with or without a newline after it, on every runner: the C++
/// finds the token again after the fill that meets the input's end, which moved the buffer.
#[test]
fn the_last_token_reads_with_no_newline_after_it() {
    let src = "let t = read<(Int, Int, Int)>()\nlet a: Int = read()\nprint(t, a)\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let (p, fs) = c.form(Form::Raw);
    let (bin, _) = cxx::build(
        c.cpp().as_bytes(),
        &["-std=c++17", "-O1", "-w", "-x", "c++"],
    )
    .expect("g++ builds it");
    let want = "7 8 9 16\n";
    for input in ["7 8 9 16", "7 8 9 16\n", "7\n8 9  16"] {
        let fixed = cpl_next_ir::run::RunInput::fixed(input);
        assert_eq!(cpl_next_interp::run(p, fs, fixed).output, want, "{input:?}");
        assert_eq!(c.run(input).output, want, "{input:?}");
        let mut ch = std::process::Command::new(&bin)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("runs");
        let mut to = ch.stdin.take().expect("stdin");
        std::io::Write::write_all(&mut to, input.as_bytes()).expect("feeds it");
        drop(to);
        let ran = ch.wait_with_output().expect("ends");
        assert!(ran.status.success(), "{input:?}");
        assert_eq!(String::from_utf8_lossy(&ran.stdout), want, "{input:?}");
    }
}

/// An event whose value traps while it is shown is not written, on the engine and in the C++:
/// the trap starts its own line, after the whole events before it.
#[test]
fn an_event_whose_value_traps_is_not_written() {
    let src = "let a = [0, 1, 2]\nlet m = a.map(x => 10 / x)\nprint(m.drop(1))\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        true,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let o = c.run("");
    assert_eq!(o.trap.as_ref().map(|t| t.code()), Some("R0203"));
    let (bin, _) = cxx::build(
        c.cpp().as_bytes(),
        &["-std=c++17", "-O1", "-w", "-pthread", "-x", "c++"],
    )
    .expect("g++ builds it");
    let ran = std::process::Command::new(bin).output().expect("runs");
    let err = String::from_utf8_lossy(&ran.stderr);
    let lines: Vec<_> = err
        .lines()
        .filter(|l| !l.starts_with("call stack: "))
        .collect();
    let (trap, evs) = lines.split_last().expect("a trap line");
    assert!(trap.starts_with("trap: R0203: line 2:"), "{err}");
    assert!(
        evs.iter()
            .all(|l| l.starts_with("ev ") && !l.contains("trap:")),
        "{err}"
    );
    assert_eq!(evs.len(), o.events.len(), "{err}");
}

/// A part the checker left untyped lowers to a trap and a value of the part's own type, so the
/// C++ builds and runs as the engine does wherever that part is not reached.
#[test]
fn an_untyped_part_is_a_value_of_its_type() {
    let src = "use std.graph.{johnson}\ndomain V = 0..2\ndomain E = 0..0\nlet es: [E](u: V, v: V, w: Int) = []\nlet d = johnson(Graph.directed(es))!\nprint(yes(d[0, 0] == Some(0)), yes(d[0, 1] == None))\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let o = c.run("");
    assert_eq!((o.output.as_str(), o.trap.is_none()), ("YES YES\n", true));
    let (bin, _) = cxx::build(
        c.cpp().as_bytes(),
        &["-std=c++17", "-O1", "-w", "-pthread", "-x", "c++"],
    )
    .expect("g++ builds it");
    let ran = std::process::Command::new(bin).output().expect("runs");
    assert_eq!(ran.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&ran.stdout), o.output);
}

/// The code a generator hands a value lent from a place binds it with the generator's marks, so
/// the calls on it are the instances the C++ passes it to.
#[test]
fn a_walk_binds_what_its_generator_yields_lent() {
    let src = "let a = [1, 1, 2, 2, 2, 1]\nfor r in a[1..].runs(): print(r.keys.first!, r.len, r[r.keys.first!])\n";
    let c = compile(
        &Std::read(),
        src,
        &Roots::std(),
        false,
        None,
        Checks::CHECKED,
    )
    .expect("compiles");
    let o = c.run("");
    assert_eq!(o.output, "1 1 1\n2 3 2\n5 1 1\n");
    let (bin, _) = cxx::build(
        c.cpp().as_bytes(),
        &["-std=c++17", "-O1", "-w", "-pthread", "-x", "c++"],
    )
    .expect("g++ builds it");
    let ran = std::process::Command::new(bin).output().expect("runs");
    assert_eq!(ran.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&ran.stdout), o.output);
}
