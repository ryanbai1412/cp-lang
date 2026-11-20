use cpl_verify::backend::{
    repo_root, scratch_dir, CommandInterpreter, CommandTranspiler, Cxx, Interpreter,
    ProtoTranspiler, Transpiler,
};
use cpl_verify::diff::{self, Campaign, Toolchain};
use cpl_verify::fuzz::eval::Trap;
use cpl_verify::fuzz::{gen, print};
use cpl_verify::{bench, layout, loc, lsp};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "\
usage: cpl-verify <command> [options]

commands:
  fuzz      run random programs through the toolchain and compare with the
            reference evaluator, under ASan and UBSan
              --seeds A..B     seeds to run (default 0..200)
              --jobs N         programs in flight (default: one per core)
              --out DIR        where failing programs are written
                               (default target/cpl-verify/failures)
  gen SEED  print the program for a seed, and with --expect its output
  layout    measure the sizes promised in a probe file
              --probe FILE     (default crates/cpl-verify/layout/core.cp)
              --gaps FILE      known misses (default: FILE with .gaps)
  loc       raw and library-stripped line counts for every port
              --ports DIR      (default proto/ports)
  bench     time check, debug run, release run and the LSP against budgets
              --runs N         runs per stage (default 5)
              --enforce        exit 1 when a stage is over budget
              --lsp CMD        language server command (default: none yet)

toolchain (fuzz, layout, bench):
  --transpile CMD   command that writes C++ for {src} to {cpp}
                    (default: python3 proto/cp.py build)
  --interp CMD      command that runs {src} (default: none yet)
  --traps LIST      runtime errors the toolchain stops on cleanly, from
                    overflow,div-by-zero,index (default: index)
  --repo DIR        repository root (default: found from the current dir)
";

struct Args(Vec<String>);

impl Args {
    fn flag(&mut self, name: &str) -> bool {
        let found = self.0.iter().position(|a| a == name);
        found.map(|i| self.0.remove(i)).is_some()
    }

    fn value(&mut self, name: &str) -> Result<Option<String>, String> {
        let Some(i) = self.0.iter().position(|a| a == name) else {
            return Ok(None);
        };
        if i + 1 >= self.0.len() {
            return Err(format!("{name} needs a value"));
        }
        self.0.remove(i);
        Ok(Some(self.0.remove(i)))
    }

    fn parsed<T: std::str::FromStr>(&mut self, name: &str, default: T) -> Result<T, String> {
        match self.value(name)? {
            Some(v) => v
                .parse()
                .map_err(|_| format!("bad value for {name}: `{v}`")),
            None => Ok(default),
        }
    }

    fn done(self) -> Result<(), String> {
        match self.0.first() {
            Some(extra) => Err(format!("unexpected argument `{extra}`")),
            None => Ok(()),
        }
    }
}

fn words(command: &str) -> Vec<String> {
    command.split_whitespace().map(str::to_string).collect()
}

struct Setup {
    root: PathBuf,
    transpiler: Box<dyn Transpiler>,
    interpreter: Option<Box<dyn Interpreter>>,
    traps: Vec<Trap>,
}

fn setup(args: &mut Args) -> Result<Setup, String> {
    let root = match args.value("--repo")? {
        Some(dir) => PathBuf::from(dir),
        None => repo_root().ok_or("cannot find the repository root; pass --repo")?,
    };
    let transpiler: Box<dyn Transpiler> = match args.value("--transpile")? {
        Some(cmd) => Box::new(CommandTranspiler {
            template: words(&cmd),
        }),
        None => Box::new(ProtoTranspiler::new(&root)),
    };
    let interpreter = args.value("--interp")?.map(|cmd| {
        Box::new(CommandInterpreter {
            template: words(&cmd),
        }) as Box<dyn Interpreter>
    });
    let traps = args
        .value("--traps")?
        .unwrap_or_else(|| "index".into())
        .split(',')
        .filter(|t| !t.is_empty())
        .map(|t| match t {
            "overflow" => Ok(Trap::Overflow),
            "div-by-zero" => Ok(Trap::DivByZero),
            "index" => Ok(Trap::IndexOutOfRange),
            other => Err(format!("unknown trap `{other}`")),
        })
        .collect::<Result<_, _>>()?;
    Ok(Setup {
        root,
        transpiler,
        interpreter,
        traps,
    })
}

fn with_pch(mut cxx: Cxx, work: &Path) -> Result<Cxx, String> {
    cxx.build_pch(&work.join("pch"))
        .map_err(|e| format!("precompiling <bits/stdc++.h>: {e}"))?;
    Ok(cxx)
}

fn fuzz(mut args: Args) -> Result<bool, String> {
    let seeds = args.value("--seeds")?.unwrap_or_else(|| "0..200".into());
    let (start, end) = seeds
        .split_once("..")
        .and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))
        .ok_or("--seeds is `start..end`")?;
    let mut campaign = Campaign::new(start..end);
    campaign.jobs = args.parsed("--jobs", campaign.jobs)?;
    let out = args.value("--out")?;
    let setup = setup(&mut args)?;
    args.done()?;
    let out = out.map_or_else(
        || setup.root.join("target/cpl-verify/failures"),
        PathBuf::from,
    );
    let work = scratch_dir("main").map_err(|e| e.to_string())?;
    let toolchain = Toolchain {
        cxx: with_pch(Cxx::sanitized(), &work)?,
        transpiler: setup.transpiler,
        interpreter: setup.interpreter,
        traps: setup.traps,
        static_refusals: vec![
            "beyond Int".into(),
            "over the 64 KB submission limit".into(),
        ],
    };
    println!(
        "fuzzing {} with {}{}, seeds {start}..{end}",
        toolchain.transpiler.name(),
        toolchain.cxx.describe(),
        toolchain
            .interpreter
            .as_ref()
            .map_or(String::new(), |i| format!(" and {}", i.name())),
    );
    let report = diff::run(&toolchain, &campaign);
    let _ = std::fs::remove_dir_all(&work);
    for failure in &report.failures {
        std::fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
        let write = |suffix: &str, text: &str| {
            let path = out.join(format!("seed-{}{suffix}.cp", failure.seed));
            std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
            Ok::<_, String>(path)
        };
        write("", &failure.source)?;
        let small = write("-min", &failure.shrunk)?;
        println!(
            "\nseed {}: {:?} in {}: {}\n{}  (written to {})",
            failure.seed,
            failure.finding.kind,
            failure.finding.path,
            failure.finding.detail,
            failure.shrunk.trim_end(),
            small.display()
        );
    }
    println!(
        "\n{} programs agreed, {} trapping programs stopped cleanly, {} refused by the checker, {} failed",
        report.programs - report.failures.len() as u64,
        report.trap_programs,
        report.refused,
        report.failures.len()
    );
    Ok(report.failures.is_empty())
}

fn gen_cmd(mut args: Args) -> Result<bool, String> {
    let expect = args.flag("--expect");
    let seed = args
        .0
        .pop()
        .and_then(|s| s.parse().ok())
        .ok_or("gen needs a seed")?;
    args.done()?;
    let case = gen::generate(seed, &gen::Config::default());
    print!("{}", print::to_source(&case.program));
    if expect {
        print!("--- expected output\n{}", case.expected);
    }
    Ok(true)
}

fn layout_cmd(mut args: Args) -> Result<bool, String> {
    let probe = args.value("--probe")?;
    let gaps = args.value("--gaps")?;
    let setup = setup(&mut args)?;
    args.done()?;
    let probe = probe.map_or_else(
        || setup.root.join("crates/cpl-verify/layout/core.cp"),
        PathBuf::from,
    );
    let gaps_path = gaps.map_or_else(|| probe.with_extension("gaps"), PathBuf::from);
    let gaps = match std::fs::read_to_string(&gaps_path) {
        Ok(text) => layout::parse_gaps(&text)?,
        Err(_) => vec![],
    };
    let work = scratch_dir("layout").map_err(|e| e.to_string())?;
    let measured = layout::audit(setup.transpiler.as_ref(), &Cxx::release(), &probe, &work);
    let _ = std::fs::remove_dir_all(&work);
    let measured = measured?;
    print!("{}", layout::report(&measured));
    let problems = layout::problems(&measured, &gaps);
    for problem in &problems {
        println!("problem: {problem}");
    }
    let off = measured.iter().filter(|m| m.size != m.probe.want).count();
    println!(
        "{} of {} sizes are as planned; {} unexpected",
        measured.len() - off,
        measured.len(),
        problems.len()
    );
    Ok(problems.is_empty())
}

fn loc_cmd(mut args: Args) -> Result<bool, String> {
    let ports = args.value("--ports")?;
    let root = args.value("--repo")?.map(PathBuf::from).or_else(repo_root);
    args.done()?;
    let ports = match (ports, root) {
        (Some(dir), _) => PathBuf::from(dir),
        (None, Some(root)) => root.join("proto/ports"),
        (None, None) => return Err("cannot find the repository root; pass --ports".into()),
    };
    print!("{}", loc::report(&loc::measure_dir(&ports)?));
    Ok(true)
}

fn bench_cmd(mut args: Args) -> Result<bool, String> {
    let runs = args.parsed("--runs", 5)?;
    let enforce = args.flag("--enforce");
    let lsp = args.value("--lsp")?.map(|cmd| words(&cmd));
    let setup = setup(&mut args)?;
    args.done()?;
    let work = scratch_dir("bench").map_err(|e| e.to_string())?;
    let release = with_pch(Cxx::release(), &work)?;
    let programs = bench::programs(
        &setup.root.join("proto/ports"),
        &setup.root.join("crates/cpl-verify/bench"),
    )?;
    let tools = bench::Tools {
        transpiler: setup.transpiler.as_ref(),
        release: &release,
        interpreter: setup.interpreter.as_deref(),
        lsp: lsp.as_deref(),
    };
    println!(
        "timing {} and {}, {runs} runs per stage",
        tools.transpiler.name(),
        release.describe()
    );
    let report = bench::run(&tools, &programs, runs, &work);
    let _ = std::fs::remove_dir_all(&work);
    print!("{}", report.render());
    Ok(!enforce || report.within_budget())
}

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    let command = args.remove(0);
    let args = Args(args);
    let result = match command.as_str() {
        "fuzz" => fuzz(args),
        "gen" => gen_cmd(args),
        "layout" => layout_cmd(args),
        "loc" => loc_cmd(args),
        "bench" => bench_cmd(args),
        // Not in the usage text: it exists to test the benchmark's client.
        "lsp-stub" => lsp::run_stub(std::io::stdin().lock(), std::io::stdout().lock())
            .map(|()| true)
            .map_err(|e| e.to_string()),
        other => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(message) => {
            eprintln!("cpl-verify: {message}");
            ExitCode::from(2)
        }
    }
}
