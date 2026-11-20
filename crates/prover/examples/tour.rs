//! A tour of the prover: `cargo run -p cpl-prover --example tour --release`.
//!
//! Each line is one question the checker asks, the answer, and how long
//! the answer took.

use std::time::Instant;

use cpl_prover::{Context, Law, Lend, Loop, Step, Structure};

fn timed<T>(label: &str, run: impl FnOnce() -> T, show: impl FnOnce(&T) -> String) {
    let start = Instant::now();
    let out = run();
    let elapsed = start.elapsed();
    let text = show(&out);
    let mut lines = text.lines();
    println!("{:>9.1?}  {label}: {}", elapsed, lines.next().unwrap_or(""));
    for line in lines {
        println!("           {line}");
    }
}

/// Names with their text: type parameters, or the fields of a structure.
type Pairs = &'static [(&'static str, &'static str)];

fn main() {
    println!("-- index bounds");
    let mut ctx = Context::new();
    ctx.var("H", "1..=1000").unwrap();
    ctx.var("W", "1..=1000").unwrap();
    ctx.var("r", "0..H").unwrap();
    ctx.var("c", "0..W").unwrap();
    for (index, range) in [
        ("r * W + c", "0..H * W"),
        ("c * W + r", "0..H * W"),
        ("r + 1", "0..H"),
    ] {
        timed(
            &format!("{index} in {range}"),
            || ctx.check_index(index, range).unwrap(),
            |v| v.to_string(),
        );
    }

    println!("-- law disjoint");
    let mut ctx = Context::new();
    ctx.var("n", "1..=2e5").unwrap();
    ctx.var("i", "0..n").unwrap();
    ctx.var("j", "0..n").unwrap();
    ctx.var("m", "0..=n").unwrap();
    let element = Law::parse("law disjoint(i, j) = i != j").unwrap();
    let slice = Law::parse("law disjoint(s, t) = s.hi <= t.lo or t.hi <= s.lo").unwrap();
    let pair = |ctx: &mut Context, a: &str, b: &str, law: &Law| {
        let (x, y) = (Lend::parse(a).unwrap(), Lend::parse(b).unwrap());
        timed(
            &format!("{a} with {b}"),
            || ctx.check_lends(&x, &y, Some(law)).unwrap(),
            |v| v.to_string(),
        );
    };
    pair(&mut ctx, "mut a[i]", "a[j]", &element);
    pair(&mut ctx, "mut a[0..m]", "mut a[m..n]", &slice);
    ctx.assume("i < j").unwrap();
    pair(&mut ctx, "mut a[i]", "a[j]", &element);
    pair(&mut ctx, "mut g.row(i)", "g.col(j)", &element);

    println!("-- overflow");
    ctx.array("a", "0..=1e9").unwrap();
    ctx.array("b", "0..=1e9").unwrap();
    timed(
        "a[i] * b[i] + n",
        || ctx.check_overflow("a[i] * b[i] + n").unwrap(),
        |findings| {
            findings
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        },
    );
    timed(
        "total += a[i] * b[i], n times",
        || ctx.check_sum("a[i] * b[i]", "n").unwrap(),
        |f| f.to_string(),
    );

    println!("-- invariants");
    let mut ctx = Context::new();
    ctx.var("n", "1..=2e5").unwrap();
    ctx.var("k", "-1e9..=1e9").unwrap();
    ctx.var("r", "0..n").unwrap();
    ctx.var("l", "0..n").unwrap();
    ctx.array("a", "-1e9..=1e9").unwrap();
    ctx.assume("l <= r").unwrap();
    let window = Loop::new("a[r] - a[l] > k")
        .unwrap()
        .invariant("l <= r")
        .unwrap()
        .step(Step::assign("l", "l + 1").unwrap());
    timed(
        "while a[r] - a[l] > k: l += 1",
        || ctx.check_loop(&window).unwrap(),
        |report| report.to_string(),
    );
    ctx.assume("k >= 0").unwrap();
    timed(
        "the same with k >= 0",
        || ctx.check_loop(&window).unwrap(),
        |report| report.to_string(),
    );

    println!("-- laws");
    let int: Pairs = &[("T", "Int")];
    let pre: Pairs = &[("T", "(sum: Int, pre: Int)")];
    let checks: [(&str, Structure, Pairs, Pairs); 5] = [
        (
            "Max",
            Structure::monoid(),
            int,
            &[("id", "Int.MIN"), ("op", "(a, b) => max(a, b)")],
        ),
        (
            "a - b",
            Structure::monoid(),
            int,
            &[("id", "0"), ("op", "(a, b) => a - b")],
        ),
        (
            "max prefix sum",
            Structure::semigroup(),
            pre,
            &[(
                "op",
                "(x, y) => (sum: x.sum + y.sum, pre: max(x.pre, x.sum + y.pre))",
            )],
        ),
        (
            "max prefix sum, with a slip",
            Structure::semigroup(),
            pre,
            &[(
                "op",
                "(x, y) => (sum: x.sum + y.sum, pre: max(x.pre, y.sum + y.pre))",
            )],
        ),
        (
            "add on max",
            Structure::action().distributive(),
            &[("T", "Int"), ("F", "Int")],
            &[
                ("id", "0"),
                ("compose", "(f, g) => f + g"),
                ("apply", "(f, x) => if x == Int.MIN: x else: x + f"),
                ("op", "(a, b) => max(a, b)"),
            ],
        ),
    ];
    for (label, structure, types, fields) in checks {
        timed(
            label,
            || structure.check(types, fields).unwrap(),
            |report| report.to_string(),
        );
    }
}
