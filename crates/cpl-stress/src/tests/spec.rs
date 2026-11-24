//! Reading the input spec from the program: loops, passes, typed reads, tags.

use super::*;

#[test]
fn reads_in_a_loop_are_generated_as_one_row_per_pass() {
    let src = "let (n, q): (1..=5, 0..=3) = read()
domain I = 1..=n
for i in I:
    let (x, y): (1..=9, 1..=9) = read()
    let w: -3..=3 = read()
    print(x + y + w + i)
for _ in 0..q:
    let k: 1..=4 = read()
    for _ in 0..n:
        let c: Char = read()
        print(c)
";
    let spec = InputSpec::from_source(src).unwrap();
    assert_eq!(spec.fields.len(), 3);
    for seed in 0..50 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap_or(-100))
            .collect();
        let (n, q) = (tokens[0], tokens[1]);
        let mut at = 2;
        for _ in 0..n {
            assert!((1..=9).contains(&tokens[at]) && (1..=9).contains(&tokens[at + 1]));
            assert!((-3..=3).contains(&tokens[at + 2]));
            at += 3;
        }
        // Letters of one pass are written together, as one word.
        let words: String = case.render().split_whitespace().skip(at).collect();
        let mut rest = words.chars();
        for _ in 0..q {
            let k = rest.next().and_then(|k| k.to_digit(10)).unwrap();
            assert!((1..=4).contains(&k));
            for _ in 0..n {
                assert!(rest.next().is_some_and(|c| c.is_ascii_lowercase()));
            }
        }
        assert_eq!(rest.next(), None, "{}", case.render());
    }
}

#[test]
fn loops_whose_reads_a_field_cannot_describe_are_refused() {
    let refused = [
        // The type uses a value worked out in the pass.
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let k: 1..=3 = read()\n    let m = k + 1\n    let a: [m]Int = read()\n",
        // The loop can stop early.
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: Int = read()\n    if a == 0: break\n",
        // A pass can skip a read.
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: Int = read()\n    if a == 0: continue\n    let b: Int = read()\n",
        // A read that is not a typed `let`.
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    print(read())\n",
        // A loop over a list, whose length is not a read.
        "for x in [1, 2]:\n    let a: Int = read()\n",
    ];
    for src in refused {
        let err = InputSpec::from_source(src).unwrap_err();
        assert!(
            err.message.contains("this loop reads"),
            "{src}: {}",
            err.message
        );
    }
    // `continue` after the last read leaves every read in place.
    let src = "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: Int = read()\n    if a == 0: continue\n    print(a)\n";
    assert!(InputSpec::from_source(src).is_ok());
}

#[test]
fn a_jump_that_ends_only_a_nested_loop_or_fn_keeps_the_pass_whole() {
    let accepted = [
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: 1..=9 = read()\n    var x = a\n    while true:\n        if x == 1: break\n        x -= 1\n    let b: 1..=9 = read()\n    print(x + b)\n",
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: 1..=9 = read()\n    for i in 0..a:\n        if i == 2: continue\n        print(i)\n    let b: 1..=9 = read()\n    print(b)\n",
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: 1..=9 = read()\n    fn f(x: Int) -> Int:\n        if x > 3: return 0\n        x\n    let b: 1..=9 = read()\n    print(f(a) + b)\n",
    ];
    for src in accepted {
        let spec = InputSpec::from_source(src).unwrap_or_else(|e| panic!("{src}: {}", e.message));
        let case = generate(&spec, &mut Rng::new(3), Size::Small(5)).unwrap();
        let tokens = case.render().split_whitespace().count() as i128;
        let n: i128 = case
            .render()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(tokens, 1 + 2 * n, "{src}");
    }
    let refused = [
        "let n: 1..=5 = read()\nfor _ in 0..n:\n    let a: 1..=9 = read()\n    for i in 0..a:\n        if i == 2: return\n    let b: 1..=9 = read()\n",
        "let n: 1..=5 = read()\n'outer: for _ in 0..n:\n    let a: 1..=9 = read()\n    for i in 0..a:\n        if i == 2: break 'outer\n    let b: 1..=9 = read()\n",
    ];
    for src in refused {
        let err = InputSpec::from_source(src).unwrap_err();
        assert!(
            err.message.contains("this loop reads"),
            "{src}: {}",
            err.message
        );
    }
}

#[test]
fn a_read_with_its_type_as_an_argument_is_generated_from_it() {
    let src = "let n: 1..=5 = read()
let a = read<[n]1..=9>()
let b = [0..n] _ => read::<-3..=-1>()
let c: [0..n]100..=200 = [0..n] _ => read()
for _ in 0..n:
    let (x, y) = read<(20..=30, 40..=50)>()
    print(x + y)
print(a, b, c)
";
    let spec = InputSpec::from_source(src).unwrap();
    assert_eq!(spec.fields.len(), 5);
    for seed in 0..30 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let n = tokens[0] as usize;
        assert_eq!(tokens.len(), 1 + 5 * n);
        let part = |at: usize| &tokens[1 + at * n..1 + (at + 1) * n];
        assert!(part(0).iter().all(|v| (1..=9).contains(v)));
        assert!(part(1).iter().all(|v| (-3..=-1).contains(v)));
        assert!(part(2).iter().all(|v| (100..=200).contains(v)));
        for row in part(3).iter().chain(part(4)).collect::<Vec<_>>().chunks(2) {
            assert!(
                (20..=30).contains(row[0]) && (40..=50).contains(row[1]),
                "{tokens:?}"
            );
        }
    }
    // A read that may run another number of times is refused, not left out.
    for src in [
        "let n: 1..=5 = read()\nprint(read() + n)\n",
        "let n: 1..=5 = read()\nlet a = [0..n] i => if i == 1: 0 else: read::<Int>()\n",
        "let n: 1..=5 = read()\nlet a = [read::<Int>() for x in 0..n if x > 1]\n",
        "let n: 1..=5 = read()\nfor a in read<[n]1..=3>():\n    print(read::<Int>())\n",
    ] {
        assert!(InputSpec::from_source(src).is_err(), "{src}");
    }
}

#[test]
fn typed_reads_inside_an_expression_are_fields_in_the_order_they_run() {
    let src = "let n: 1..=5 = read()
print(read::<1..=3>() + n * read::<7..=8>())
let b = [0..n] _ => (read::<10..=19>(), read::<20..=29>())
for a in read<[n]30..=39>():
    print(a)
var at: [0..n]Int = 0
for i in 0..n:
    at[read<0..n>()] = i
for _ in 0..n:
    match read::<40..=41>():
        40 => print(1)
        _ => print(2)
";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..30 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let n = tokens[0] as usize;
        assert_eq!(tokens.len(), 3 + 5 * n, "{tokens:?}");
        assert!((1..=3).contains(&tokens[1]) && (7..=8).contains(&tokens[2]));
        for (at, pair) in tokens[3..3 + 2 * n].chunks(2).enumerate() {
            assert!(
                (10..=19).contains(&pair[0]) && (20..=29).contains(&pair[1]),
                "{at}"
            );
        }
        assert!(tokens[3 + 2 * n..3 + 3 * n]
            .iter()
            .all(|v| (30..=39).contains(v)));
        assert!(tokens[3 + 3 * n..3 + 4 * n]
            .iter()
            .all(|v| (0..n as i128).contains(v)));
        assert!(tokens[3 + 4 * n..].iter().all(|v| (40..=41).contains(v)));
    }
}

#[test]
fn a_pass_reads_a_count_then_that_many_values() {
    let src = "let n: 1..=5 = read()
for _ in 0..n:
    let k: 0..=4 = read()
    let a: [k]1..=9 = read()
    for _ in 0..k:
        let (lo, c): (0..=k, 0..=k) = read()
        print(lo + c)
    print(a)
";
    let spec = InputSpec::from_source(src).unwrap();
    assert_eq!(spec.fields.len(), 2);
    let tokens = |case: &Case| -> Vec<i128> {
        case.render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect()
    };
    let rows = |tokens: &[i128]| {
        let mut at = 1;
        let mut rows = Vec::new();
        for _ in 0..tokens[0] {
            let k = tokens[at];
            let a = tokens[at + 1..][..k as usize].to_vec();
            let pairs = tokens[at + 1 + k as usize..][..2 * k as usize].to_vec();
            at += 1 + 3 * k as usize;
            rows.push((k, a, pairs));
        }
        assert_eq!(at, tokens.len(), "{tokens:?}");
        rows
    };
    let mut counts = HashSet::new();
    for seed in 0..100 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        for (k, a, pairs) in rows(&tokens(&case)) {
            assert!((0..=4).contains(&k));
            assert!(a.iter().all(|v| (1..=9).contains(v)));
            assert!(pairs.iter().all(|v| (0..=k).contains(v)));
            counts.insert(k);
        }
    }
    assert_eq!(counts.len(), 5, "every count is made: {counts:?}");
    // A 7 anywhere fails: the pass with it shrinks to one value, its
    // count with it, and the other passes go.
    let fails = |case: &Case| rows(&tokens(case)).iter().any(|(_, a, _)| a.contains(&7));
    let start = (0..)
        .map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap())
        .find(|case| fails(case) && tokens(case).len() > 8)
        .unwrap();
    let smallest = shrink(&spec, &start, 2000, &mut |case: &Case| fails(case));
    assert_eq!(
        tokens(&smallest),
        [1, 1, 7, 0, 0],
        "from {:?}",
        tokens(&start)
    );
}

#[test]
fn a_value_and_an_element_go_together_when_neither_goes_alone() {
    // A FIFO memory of `m` words that holds only `m - 1`: failing needs a
    // word read again `m - 1` new words later, so neither `m` nor the
    // words shrink alone.
    let src = "let (m, n): (1..=100, 1..=1000) = read()
for _ in 0..n:
    let w: 0..=1000 = read()
    print(w)
";
    let spec = InputSpec::from_source(src).unwrap();
    let misses = |m: usize, words: &[i128], room: usize| {
        let mut memory: Vec<i128> = Vec::new();
        let mut count = 0;
        for &w in words {
            if memory.contains(&w) {
                continue;
            }
            count += 1;
            memory.push(w);
            if memory.len() > room.min(m) {
                memory.remove(0);
            }
        }
        count
    };
    let fails = |case: &Case| {
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let m = tokens[0] as usize;
        let words = &tokens[2..];
        misses(m, words, m) != misses(m, words, m - 1)
    };
    let start = Case {
        values: vec![
            Value::Tuple(vec![Value::Int(4), Value::Int(5)]),
            Value::Seq([4, 0, 1, 2, 4].map(Value::Int).to_vec()),
        ],
        cases: Vec::new(),
        until_eof: false,
    };
    assert!(fails(&start));
    let smallest = shrink(&spec, &start, 2000, &mut |case: &Case| fails(case));
    assert_eq!(smallest.render(), "1 2\n0 0\n");
}

#[test]
fn reads_no_field_describes_are_refused_not_left_out() {
    let cases = [
        ("let n: 1..=5 = read()\nfn row() -> Int:\n    let x: Int = read()\n    if x > 0: return x\n    x\nprint(row())\n", "this fn reads input"),
        ("let n: 1..=5 = read()\n@memo\nfn row(i: Int) -> Int:\n    let x: Int = read()\n    x\nprint(row(n))\n", "this fn reads input"),
        (
            "@!multitest\nlet n: 2..=9 = read()\nfn pick() -> Int: n\nlet x: 1..=n = read(gen: pick)\n",
            "read(gen: ..)",
        ),
        ("let n: 1..=5 = read()\nfor _ in 0..n:\n    let x: ?Int = read(none: \"-\")\n", "this loop reads"),
    ];
    for (src, wanted) in cases {
        let err = InputSpec::from_source(src).unwrap_err();
        assert!(err.message.contains(wanted), "{src}: {}", err.message);
    }
    // `read2(n)` is a call of the program's own fn, not of `read`.
    let src = "let n: 1..=5 = read()\nfn read2(x: Int) -> Int: x\nprint(read2(n))\n";
    assert!(InputSpec::from_source(src).is_ok());
}

#[test]
fn a_domain_read_as_a_size_is_generated_with_its_arrays() {
    let src = "domain V = read(1..=6)
let p: [V]V = read()
let t: 1..=3 = read()
for _ in 0..t:
    domain S = read(0..=4)
    let s: [S]V = read()
    print(s)
print(p)
";
    let spec = InputSpec::from_source(src).unwrap();
    let mut sizes = HashSet::new();
    for seed in 0..100 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let n = tokens[0];
        assert!((1..=6).contains(&n));
        let p = &tokens[1..][..n as usize];
        assert!(p.iter().all(|v| (0..n).contains(v)), "{tokens:?}");
        let mut at = 1 + n as usize;
        for _ in 0..tokens[at] {
            at += 1;
            let k = tokens[at];
            assert!((0..=4).contains(&k));
            let s = &tokens[at + 1..][..k as usize];
            assert!(s.iter().all(|v| (0..n).contains(v)), "{tokens:?}");
            at += k as usize;
        }
        assert_eq!(at + 1, tokens.len(), "{tokens:?}");
        sizes.insert(n);
    }
    assert!(sizes.len() > 3);
}

#[test]
fn a_fn_that_reads_is_read_where_each_call_is() {
    let src = "fn read_pair() -> (Int, Int):
    let a: 1..=3 = read()
    let b: 7..=8 = read()
    (a, b)

fn solve():
    let p = read_pair()
    let k: 20..=29 = read()
    print(p.0 + k)

let first = read_pair()
solve()
";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len(), 5, "{tokens:?}");
        assert!((1..=3).contains(&tokens[0]) && (7..=8).contains(&tokens[1]));
        assert!((1..=3).contains(&tokens[2]) && (7..=8).contains(&tokens[3]));
        assert!((20..=29).contains(&tokens[4]), "{tokens:?}");
    }
    // A call that runs only some of the time, a reader with a parameter
    // or an early `return` reads what no field can describe.
    for refused in [
        "fn one() -> Int:\n    let a: 1..=3 = read()\n    a\nlet c: 0..=1 = read()\nif c == 1: print(one())\n",
        "fn one(m: Int) -> Int:\n    let a: 1..=3 = read()\n    a + m\nlet b = one(2)\n",
        "fn one() -> Int:\n    let c: 0..=1 = read()\n    if c == 0: return 0\n    let a: 1..=3 = read()\n    a\nlet b = one()\n",
    ] {
        let error = InputSpec::from_source(refused).unwrap_err();
        assert!(
            ["this fn reads input", "is not a top-level"].iter().any(|m| error.message.contains(m)),
            "{refused}: {}",
            error.message
        );
    }
}

#[test]
fn a_fn_that_reads_fixed_values_reads_them_in_any_expression() {
    let src = "fn ri() -> Int:
    let x: 1..=9 = read()
    return x

let n: 1..=4 = read()
var a = List<Int>()
for _ in 0..n: a.push(ri() + ri())
print(a.sum())
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("value-reader-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len() as i128, 1 + 2 * tokens[0], "{text}");
        assert!(tokens[1..].iter().all(|t| (1..=9).contains(t)), "{text}");
    }
    std::fs::remove_file(&path).ok();
    // Called only some of the time, its reads are not a field.
    let branch = "fn ri() -> Int:\n    let x: Int = read()\n    x\nlet c: 0..=1 = read()\nif c == 1: print(ri())\n";
    assert!(InputSpec::from_source(branch).is_err());
    // A type that uses a value the body read is not the same each call.
    let dependent = "fn row() -> Int:\n    let k: 1..=3 = read()\n    let a: [k]Int = read()\n    a.sum()\nfor _ in 0..2: print(row())\n";
    assert!(InputSpec::from_source(dependent).is_err());
}

#[test]
fn a_write_of_read_into_a_typed_array_reads_its_element_type() {
    let src = "let n: 1..=5 = read()
var a: [0..=n]0..=9 = 0
for i in 1..=n: a[i] = read()
var grid: [0..n, 0..n]Int = 0
for i in 0..n:
    for j in 0..=i: grid[i, j] = read()
print(a.sum() + grid[n - 1, 0])
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("elem-read-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let words: Vec<i128> = text
            .split_whitespace()
            .map(|w| w.parse().unwrap())
            .collect();
        let n = words[0];
        assert_eq!(words.len() as i128, 1 + n + n * (n + 1) / 2, "{text}");
        assert!(
            words[1..=n as usize].iter().all(|v| (0..=9).contains(v)),
            "{text}"
        );
    }
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_loop_read_may_use_the_loop_name() {
    let src = "let n: 1..=5 = read()
var best = 0
for r in 1..=n:
    let row: [0..r]0..=9 = read()
    best = max(best, row.sum())
print(best)
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("loop-index-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let n: i128 = text.split_whitespace().next().unwrap().parse().unwrap();
        assert_eq!(
            text.split_whitespace().count() as i128,
            1 + n * (n + 1) / 2,
            "{text}"
        );
        // Shrinking keeps row `r` at `r` values.
        let shrunk = crate::shrink::fit(&spec, &case).unwrap().render();
        assert_eq!(
            shrunk.split_whitespace().count(),
            text.split_whitespace().count()
        );
    }
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_word_tag_chooses_by_the_words_the_program_compares_it_with() {
    let src = "let q: 1..=6 = read()
var total = 0
for _ in 0..q:
    let op: Str = read()
    if op == \"ADD\":
        let x: 1..=9 = read()
        total += x
    elif op == \"SHOW\": print(total)
    else:
        let (a, b): (1..=9, 1..=9) = read()
        total -= a * b
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("word-tag-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    let mut seen = HashSet::new();
    for seed in 0..60 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(6)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let mut tokens = text.split_whitespace();
        for _ in 0..tokens.next().unwrap().parse::<usize>().unwrap() {
            let op = tokens.next().unwrap();
            seen.insert(op.to_string());
            let more = match op {
                "ADD" => 1,
                "SHOW" => 0,
                _ => 2,
            };
            for _ in 0..more {
                assert!(
                    (1..=9).contains(&tokens.next().unwrap().parse::<i128>().unwrap()),
                    "{text}"
                );
            }
        }
        assert!(tokens.next().is_none(), "{text}");
        let fitted = crate::shrink::fit(&spec, &case).unwrap();
        assert_eq!(fitted.render(), text);
    }
    assert_eq!(seen.len(), 3, "{seen:?}");
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_reader_may_return_once_its_reads_are_done() {
    let src = "fn main_solve():
    let n: 1..=4 = read()
    let xs: [0..n]0..=9 = read()
    fn deep(i: Int) -> Int:
        if i == n: return 0
        xs[i] + deep(i + 1)
    if deep(0) == 0:
        print(\"none\")
        return
    print(deep(0))

main_solve()
";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(4)).unwrap();
        let text = case.render();
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len() as i128, tokens[0] + 1, "{text}");
    }
}

#[test]
fn floats_and_wide_ints_are_read_as_whole_numbers() {
    let src = "let (k, n): (F64, 1..=5) = read()
let xs: [0..n]F64 = read()
let w: I128 = read()
print(k + xs.sum(), w)
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("floats-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len() as i128, tokens[1] + 3, "{text}");
    }
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_tag_read_first_in_a_tuple_or_above_other_reads_still_chooses() {
    let src = "domain Cell = 1..=9
let q: 1..=6 = read()
for _ in 0..q:
    let (kind, l): (0..=2, Cell) = read()
    let r: Cell = read()
    match kind:
        1 =>
            let (a, b): (Cell, Cell) = read()
            print(a + b + l + r)
        _ => print(l + r)
";
    let spec = InputSpec::from_source(src).unwrap();
    let mut kinds = HashSet::new();
    for seed in 0..100 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        let mut tokens = text.split_whitespace().map(|t| t.parse::<i128>().unwrap());
        for _ in 0..tokens.next().unwrap() {
            let kind = tokens.next().unwrap();
            assert!((0..=2).contains(&kind), "{text}");
            kinds.insert(kind);
            for _ in 0..if kind == 1 { 4 } else { 2 } {
                assert!((1..=9).contains(&tokens.next().unwrap()), "{text}");
            }
        }
        assert!(tokens.next().is_none(), "{text}");
    }
    assert_eq!(kinds.len(), 3);
    let shadowed = "let (kind, l): (0..=1, 0..=1) = read()
let kind: 5..=6 = read()
if kind == 5:
    let x: Int = read()
    print(x)
";
    assert!(InputSpec::from_source(shadowed).is_err());
}

#[test]
fn a_query_reads_what_the_branch_its_tag_takes_reads() {
    let src = "let q: 1..=6 = read()
for _ in 0..q:
    let op: Int = read()
    if op == 1:
        let (l, r): (1..=9, 1..=9) = read()
        print(l + r)
    elif op == 2 or op == 3:
        let i: 1..=9 = read()
        print(i)
    else:
        print(0)
    let kind: Char = read()
    match kind:
        'B' =>
            let k: 0..=3 = read()
            let a: [k]5..=7 = read()
            print(a)
        _ => print(kind)
";
    let spec = InputSpec::from_source(src).unwrap();
    let mut seen = HashSet::new();
    for seed in 0..200 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        let mut tokens = text.split_whitespace();
        let mut next = || tokens.next().expect("a token");
        for _ in 0..next().parse::<usize>().unwrap() {
            let op: i128 = next().parse().unwrap();
            let reads = match op {
                1 => 2,
                2 | 3 => 1,
                _ => 0,
            };
            for _ in 0..reads {
                assert!((1..=9).contains(&next().parse::<i128>().unwrap()), "{text}");
            }
            let kind = next();
            if kind == "B" {
                let k: usize = next().parse().unwrap();
                for _ in 0..k {
                    assert!((5..=7).contains(&next().parse::<i128>().unwrap()), "{text}");
                }
            }
            seen.insert((op.clamp(0, 4), kind == "B"));
        }
        assert!(tokens.next().is_none(), "{text}");
    }
    assert!(seen.len() >= 6, "{seen:?}");
    // Fails when some query asks for `B` with a 7 in it: the smallest
    // such input is one query that reads nothing, then a `B` with one 7.
    let mut fails = |case: &Case| {
        let Value::Seq(rows) = &case.values[1] else {
            return false;
        };
        rows.iter().any(|row| {
            matches!(row, Value::Tuple(parts) if matches!(&parts[1],
                Value::Tuple(b) if b.first() == Some(&Value::Char(i128::from(b'B')))
                    && matches!(b.get(2), Some(Value::Seq(a)) if a.contains(&Value::Int(7)))))
        })
    };
    let start = (0..500)
        .filter_map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)))
        .find(|case| case.weight().0 >= 12 && fails(case))
        .expect("a large failing case");
    let smallest = shrink(&spec, &start, 4000, &mut fails);
    assert_eq!(
        smallest.render(),
        "1\n4\nB\n1\n7\n",
        "from {}",
        start.render()
    );
}

#[test]
fn a_read_in_one_comparison_is_read() {
    let src = "let n: 1..=4 = read()
domain V = 0..n
let lit: [V]Bool = [V] _ => read::<0..=1>() == 1
print(lit.count(true))
";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..10 {
        let text = generate(&spec, &mut Rng::new(seed), Size::Small(4))
            .unwrap()
            .render();
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len() as i128, 1 + tokens[0], "{text}");
        assert!(tokens[1..].iter().all(|v| (0..=1).contains(v)), "{text}");
    }
    // A chain may stop early, so how many of its reads run is unknown.
    assert!(
        InputSpec::from_source("let a = 0 < read::<Int>() < read::<Int>()\nprint(a)\n").is_err()
    );
}

#[test]
fn bare_reads_take_the_type_the_call_they_are_passed_to_wants() {
    let src = "struct Seg:
    l: 1..=9
    r: 1..=9
let (n, m): (1..=5, 0..=6) = read()
domain V = 1..=n
domain E = 0..m
let g = Graph<V, E>.undirected(read())
let s = Seg(read(), read())
print(g.deg(V.first) + s.l + s.r)
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("bare-reads-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    for seed in 0..30 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let (n, m) = (tokens[0], tokens[1]);
        assert_eq!(tokens.len() as i128, 2 + 2 * m + 2, "{text}");
        let edges = &tokens[2..2 + 2 * m as usize];
        assert!(edges.iter().all(|v| (1..=n).contains(v)), "{text}");
        assert!(
            tokens[tokens.len() - 2..]
                .iter()
                .all(|v| (1..=9).contains(v)),
            "{text}"
        );
    }
    std::fs::remove_file(&path).ok();
}

#[test]
fn stdlib_structs_residues_and_unsigned_values_are_generated() {
    let src = "const P = 7
let p: Pt = read()
let q: Pt3 = read()
let r: Mod<P> = read()
let s: Mod<5> = read()
let u: U64 = read()
print(p.x + p.y + q.z, r, s, u)
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("std-structs-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    for seed in 0..40 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len(), 8, "{text}");
        assert!(
            (0..7).contains(&tokens[5]) && (0..5).contains(&tokens[6]),
            "{text}"
        );
        assert!(tokens[7] >= 0, "{text}");
    }
    let min = shrink(
        &spec,
        &generate(&spec, &mut Rng::new(1), Size::Small(8)).unwrap(),
        2000,
        &mut |_| true,
    );
    let text = min.render();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[2..], ["0", "0", "0"]);
    let shadowed = "struct Pt(x: 1..=2)\nlet p: Pt = read()\nprint(p.x)\n";
    let case = generate(
        &InputSpec::from_source(shadowed).unwrap(),
        &mut Rng::new(3),
        Size::Small(8),
    )
    .unwrap();
    assert!(
        ["1\n", "2\n"].contains(&case.render().as_str()),
        "{}",
        case.render()
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn structs_and_tagged_enums_are_read_field_by_field() {
    let src = "domain Row = 1..=3
struct Pt(x: 0..=9, y: Row)
@tags(\"R\", \"C\")
enum Stroke:
    Across(r: Row)
    Down(c: 1..=4, d: 0..=1)
let n: 1..=5 = read()
let ps: [n]Pt = read()
let k: 1..=5 = read()
let s: [k]Stroke = read()
for p in ps:
    print(p.x + p.y)
for x in s:
    match x:
        Across(r) => print(r)
        Down(c, d) => print(c * 10 + d)
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("records-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    let mut tags = HashSet::new();
    for seed in 0..40 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        tags.extend(
            text.split_whitespace()
                .filter(|t| *t == "R" || *t == "C")
                .map(String::from),
        );
    }
    assert_eq!(tags.len(), 2);
    let fails = |case: &Case| case.render().split_whitespace().any(|t| t == "C");
    let start = (0..)
        .map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap())
        .find(|case| fails(case))
        .unwrap();
    let min = shrink(&spec, &start, 2000, &mut |case| fails(case));
    assert_eq!(min.render(), "1\n0 1\n1\nC 1 0\n");
    std::fs::remove_file(&path).ok();
}

#[test]
fn an_optional_read_with_none_written_as_a_number_is_generated_both_ways() {
    let src = "let n: 1..=6 = read()
domain P = 1..=n
let a: [P]?(1..=n) = read(none: 0)
let b: ?(5..=9) = read(none: \"-1\")
var placed = 0
for i in P:
    if let v = a[i]:
        placed += v
if let x = b:
    placed += x
print(placed)
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("maybe-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    let (mut absent, mut present) = (0, 0);
    for seed in 0..40 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let n = tokens[0];
        for v in &tokens[1..=n as usize] {
            assert!((0..=n).contains(v), "{text}");
            if *v == 0 {
                absent += 1
            } else {
                present += 1
            }
        }
        assert!(tokens[1 + n as usize] == -1 || (5..=9).contains(&tokens[1 + n as usize]));
    }
    assert!(absent > 10 && present > 10, "{absent} {present}");
    // A value that is there but no longer needed shrinks to `none`.
    let fails = |case: &Case| {
        case.render()
            .lines()
            .nth(1)
            .is_some_and(|a| a.split(' ').any(|t| t == "3"))
    };
    let start = (0..)
        .map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap())
        .find(|case| fails(case))
        .unwrap();
    let min = shrink(&spec, &start, 2000, &mut |case| fails(case));
    assert_eq!(min.render(), "3\n0 0 3\n-1\n");
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_read_with_gen_draws_what_its_fn_makes_from_the_input_above() {
    let src = "let n: 3..=8 = read()
domain I = 0..n
fn word() -> [I]Char: [I] _ => 'a' + rand(0, 2)
let s: [I]Char = read(gen: word)
let k: 1..=n = read()
var c = 0
for i in I:
    if s[i] == 'c':
        c += 1
print(c + k)
";
    let spec = InputSpec::from_source(src).unwrap();
    let path = std::env::temp_dir().join(format!("drawn-{}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    let mut words = HashSet::new();
    for seed in 0..20 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let text = case.render();
        assert_eq!(crate::explain::runtime_error(&path, &text), None, "{text}");
        let lines: Vec<&str> = text.lines().collect();
        let n: usize = lines[0].parse().unwrap();
        assert_eq!(lines[1].len(), n, "{text}");
        assert!(
            lines[1].bytes().all(|b| (b'a'..=b'c').contains(&b)),
            "{text}"
        );
        let k: usize = lines[2].parse().unwrap();
        assert!((1..=n).contains(&k), "{text}");
        words.insert(lines[1].to_string());
    }
    assert!(words.len() > 10, "{words:?}");
    let fails = |case: &Case| {
        case.render()
            .lines()
            .nth(1)
            .is_some_and(|s| s.contains("cc"))
    };
    let start = (0..)
        .map(|seed| generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap())
        .find(|case| fails(case) && case.render().lines().next() != Some("3"))
        .unwrap();
    let min = shrink(&spec, &start, 2000, &mut |case| fails(case));
    let text = min.render();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(
        (lines[0], lines[1].len(), lines[2]),
        ("3", 3, "1"),
        "{}",
        min.render()
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn a_swapped_in_brute_builds_once_and_leaves_no_file_beside_the_source() {
    if std::process::Command::new("g++")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let dir = scratch("part-brute");
    let file = dir.join("p.cp");
    let src = "fn f(n: Int) -> Int = n * 2\n\n@alt(f)\nfn f_slow(n: Int) -> Int = n + n\n\nlet n: 1..=9 = read()\nprint(f(n))\n";
    std::fs::write(&file, src).unwrap();
    let swapped = crate::cli::swap_in_part_brutes(src).unwrap();
    let first = crate::cli::build_part_brute(&file, "p.cp", &swapped).unwrap();
    let again = crate::cli::build_part_brute(&file, "p.cp", &swapped).unwrap();
    assert_eq!(first.argv, again.argv);
    // Named for the source alone: a name that changed per run (say, with
    // the process id) would change the C++ and miss the build cache.
    let binary = first.argv.iter().find(|a| a.contains(".p.brute")).unwrap();
    let name = Path::new(binary).file_name().unwrap().to_str().unwrap();
    let hashes = name.strip_prefix(".p.brute-").unwrap().split('-');
    assert!(
        hashes
            .into_iter()
            .all(|h| h.len() == 16 && h.chars().all(|c| c.is_ascii_hexdigit())),
        "{name}"
    );
    let outcome = first.run("4\n", Duration::from_secs(10)).unwrap();
    assert_eq!(outcome.stdout, "8\n");
    let left: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(left, ["p.cp"]);
}

#[test]
fn a_pass_that_skips_the_first_or_last_index_reads_from_the_rest() {
    let src = "let n: 1..=6 = read()
domain V = 1..=n
var up: [V]Int = 0
for v in V:
    if v == V.first: continue
    let b: 0..=9 = read()
    up[v] = b
for i in 0..n:
    if i == n - 1: continue
    let c: 0..=9 = read()
    up[V.first] += c
print(up[V.first])
";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..10 {
        let text = generate(&spec, &mut Rng::new(seed), Size::Small(6))
            .unwrap()
            .render();
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len() as i128, 1 + 2 * (tokens[0] - 1), "{text}");
    }
    // Skipping a pass in the middle leaves a row out of the count.
    let middle = "let n: 1..=6 = read()\nfor i in 0..n:\n    if i == 2: continue\n    let c: Int = read()\n    print(c)\n";
    assert!(InputSpec::from_source(middle).is_err());
}

#[test]
fn a_domain_a_pass_computes_refuses_only_the_reads_that_use_it() {
    let src = "let q: 1..=4 = read()
for _ in 0..q:
    let k: 1..=5 = read()
    let w = k * 2
    domain W = 0..w
    let picks: [k]0..=9 = read()
    print(picks.sum + W.size)
";
    let spec = InputSpec::from_source(src).unwrap();
    let text = generate(&spec, &mut Rng::new(1), Size::Small(4))
        .unwrap()
        .render();
    assert!(!text.is_empty());
    let uses = "let q: 1..=4 = read()
for _ in 0..q:
    let k: 1..=5 = read()
    let w = k * 2
    domain W = 0..w
    let p: W = read()
    print(p)
";
    assert!(InputSpec::from_source(uses).is_err());
}

#[test]
fn a_write_of_read_into_a_typed_list_reads_its_element_type() {
    let src = "let n: 1..=5 = read()
var cur: List<0..=9> = List.filled(n * (n + 1) / 2, 0)
for i in 0..n:
    for j in 0..=i:
        cur[i * (i + 1) / 2 + j] = read()
print(cur.sum)
";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..10 {
        let text = generate(&spec, &mut Rng::new(seed), Size::Small(5))
            .unwrap()
            .render();
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let n = tokens[0];
        assert_eq!(tokens.len() as i128, 1 + n * (n + 1) / 2, "{text}");
        assert!(tokens[1..].iter().all(|v| (0..=9).contains(v)), "{text}");
    }
}

#[test]
fn a_table_that_skips_its_first_cell_reads_the_rest() {
    let src = "let n: 1..=6 = read()
domain V = 1..=n
let par = [0..n] i => if i == 0: 0 else: read::<0..=9>()
let up: [V]Int = [V] v => if v == V.last: 0 else: read::<0..=9>()
print(par.sum + up.sum)
";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..10 {
        let text = generate(&spec, &mut Rng::new(seed), Size::Small(6))
            .unwrap()
            .render();
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len() as i128, 1 + 2 * (tokens[0] - 1), "{text}");
    }
    let middle = "let n: 1..=6 = read()\nlet a = [0..n] i => if i == 2: 0 else: read::<Int>()\nprint(a.sum)\n";
    assert!(InputSpec::from_source(middle).is_err());
}

#[test]
fn a_list_of_bare_reads_reads_the_element_type_it_is_declared_with() {
    let src = "let n: 1..=6 = read()
let a: List<0..=9> = [read() for _ in 0..n]
print(a.sum)
";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..10 {
        let text = generate(&spec, &mut Rng::new(seed), Size::Small(6))
            .unwrap()
            .render();
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert_eq!(tokens.len() as i128, 1 + tokens[0], "{text}");
        assert!(tokens[1..].iter().all(|v| (0..=9).contains(v)), "{text}");
    }
    let filtered =
        "let n: 1..=6 = read()\nlet a: List<Int> = [read() for i in 0..n if i > 1]\nprint(a.sum)\n";
    assert!(InputSpec::from_source(filtered).is_err());
}

#[test]
fn a_top_level_branch_on_a_value_read_above_chooses_what_follows() {
    let src = "let k: 1..=2 = read()
let n: 1..=4 = read()
if k == 2:
    let a: [n]0..=9 = read()
    print(a.sum)
else:
    let b: 5..=7 = read()
    print(b * n)
";
    let spec = InputSpec::from_source(src).unwrap();
    let mut seen = HashSet::new();
    for seed in 0..30 {
        let text = generate(&spec, &mut Rng::new(seed), Size::Small(4))
            .unwrap()
            .render();
        let tokens: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let (k, n) = (tokens[0], tokens[1]);
        seen.insert(k);
        match k {
            2 => assert_eq!(tokens.len() as i128, 2 + n, "{text}"),
            _ => assert!(tokens.len() == 3 && (5..=7).contains(&tokens[2]), "{text}"),
        }
    }
    assert_eq!(seen.len(), 2);
    // A read after the branch could use what it moved into the arms.
    let after = "let k: 1..=2 = read()\nif k == 2:\n    let a: Int = read()\n    print(a)\nlet m: Int = read()\nprint(m)\n";
    assert!(InputSpec::from_source(after).is_err());
}

#[test]
fn interactor_after_top_level_fn_still_sees_multitest_divider() {
    let src = "\
@!interactive
fn helper(x: Int) -> Int = x

@!multitest
let n: 1..=5 = read()
print(helper(n))

@interactor
fn judge(mut sol: Interactor):
    let n: 1..=5 = read()
    let secret: 1..=n = read()
    sol.send(n)
    let got: Int = sol.recv()
    assert got == secret
";
    let spec = InputSpec::from_source(src).unwrap();
    let per_case = spec.cases.as_ref().expect("multitest divider must be seen");
    assert_eq!(per_case.fields.len(), 2);
}

#[test]
fn bounds_written_with_powers_and_shifts_are_generated() {
    let src = "let n: 1..=10**12 = read()
let k: 0..=4 = read()
let w: [0..(1 << k)]Int = read()
let r: 0..=(n % 7 + (8 >> 1)) = read()
print(n + k + w.len + r)
";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..30 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(8)).unwrap();
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let (n, k) = (tokens[0], tokens[1]);
        assert!((1..=10i128.pow(12)).contains(&n) && (0..=4).contains(&k));
        let r = tokens[2 + (1usize << k)];
        assert!((0..=n % 7 + 4).contains(&r), "{}", case.render());
        assert_eq!(tokens.len(), 3 + (1 << k), "{}", case.render());
    }
}

#[test]
fn a_read_of_a_range_is_a_field_of_that_range() {
    let src = "let n: 1..=5 = read()\nlet k = read(0..10)\nprint(n + k)\n";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..30 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Full).unwrap();
        let tokens: Vec<i128> = case
            .render()
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert!(matches!(tokens[..], [n, k] if (1..=5).contains(&n) && (0..10).contains(&k)));
    }
}

#[test]
fn a_constrained_str_is_a_word_over_its_alphabet_of_a_length_in_its_range() {
    let src = "let n: 2..=4 = read()\nlet s: Str(len: n..=n) of \"ABC\" = read()\nlet t: Str of \"01\" = read()\n";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..30 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(6)).unwrap();
        let text = case.render();
        let tokens: Vec<&str> = text.split_whitespace().collect();
        let [n, s, t] = tokens[..] else {
            panic!("{text}")
        };
        assert_eq!(s.len(), n.parse::<usize>().unwrap(), "{text}");
        assert!(s.bytes().all(|b| b"ABC".contains(&b)), "{text}");
        assert!(
            !t.is_empty() && t.bytes().all(|b| b"01".contains(&b)),
            "{text}"
        );
    }
}

#[test]
fn an_enum_reads_its_variants_tag_or_else_its_name() {
    let src = "enum Op = @tag(\"add\") Add(x: 1..=9) | Del | @tag(7) Seven\nlet q: 1..=6 = read()\nlet ops: [q]Op = read()\n";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..30 {
        let text = generate(&spec, &mut Rng::new(seed), Size::Small(6))
            .unwrap()
            .render();
        let mut tokens = text.split_whitespace().skip(1);
        while let Some(tag) = tokens.next() {
            match tag {
                "add" => assert!((1..=9).contains(&tokens.next().unwrap().parse::<i32>().unwrap())),
                "Del" | "7" => {}
                other => panic!("{other} in {text}"),
            }
        }
    }
}

#[test]
fn a_range_reads_two_ends_in_order() {
    let src = "let n: 1..=9 = read()\ndomain V = 1..=n\nlet r: Range<V> = read()\n";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..30 {
        let text = generate(&spec, &mut Rng::new(seed), Size::Small(9))
            .unwrap()
            .render();
        let t: Vec<i128> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        assert!(
            matches!(t[..], [n, l, r] if 1 <= l && l <= r && r <= n),
            "{text}"
        );
    }
}

#[test]
fn derived_domains_counted_lists_and_literal_unions_are_generated() {
    let src = "let n: 1..=4 = read()
let m: 1..=3 = read()
domain R = [n]
domain C = [m]
domain Cell = (R, C)
enum Dir:
    L
    U
    D
let g: [Cell]0..=9 = read()
let at: Cell = read()
let lists: [2]List<5..=7> = read()
let op: \"add\" | \"del\" = read()
let open: 3.. = read()
let a: [R.cuts, Dir]Int = read()
let b: [Pair<R>]Int = read()
let c: [Subset<C>]Int = read()
";
    let spec = InputSpec::from_source(src).unwrap();
    for seed in 0..30 {
        let case = generate(&spec, &mut Rng::new(seed), Size::Small(5)).unwrap();
        let text = case.render();
        let tokens: Vec<&str> = text.split_whitespace().collect();
        let mut at = 0;
        let mut next = || {
            at += 1;
            tokens[at - 1]
        };
        let n: usize = next().parse().unwrap();
        let m: usize = next().parse().unwrap();
        for _ in 0..n * m {
            assert!((0..=9).contains(&next().parse::<i32>().unwrap()), "{text}");
        }
        assert!(next().parse::<usize>().unwrap() < n, "{text}");
        assert!(next().parse::<usize>().unwrap() < m, "{text}");
        for _ in 0..2 {
            let k: usize = next().parse().unwrap();
            for _ in 0..k {
                assert!((5..=7).contains(&next().parse::<i32>().unwrap()), "{text}");
            }
        }
        assert!(["add", "del"].contains(&next()), "{text}");
        assert!(next().parse::<i128>().unwrap() >= 3, "{text}");
        let rest = (n + 1) * 3 + n * (n - 1) / 2 + (1 << m);
        assert_eq!(tokens.len() - at, rest, "{text}");
    }
}

#[test]
fn sentinel_ended_loops_end_with_their_sentinel_only() {
    let pairs = "while true:
    let (a, b) = read<(0..=3, 0..=3)>()
    if a == 0 and b == 0:
        break
    domain R = 0..a
    let row = read<[R]1..=2>()
    print(a + b + row.len)
";
    let commands = "enum Cmd:
    @tag(\"1\") Add(x: 1..=9)
    @tag(\"2\") Stop
while true:
    match read<Cmd>():
        Add(x) => print(x)
        Stop => break
";
    for seed in 0..30 {
        let spec = InputSpec::from_source(pairs).unwrap();
        let text = generate(&spec, &mut Rng::new(seed), Size::Small(5))
            .unwrap()
            .render();
        let tokens: Vec<i32> = text
            .split_whitespace()
            .map(|t| t.parse().unwrap())
            .collect();
        let mut at = 0;
        while (tokens[at], tokens[at + 1]) != (0, 0) {
            let a = tokens[at] as usize;
            assert!(
                tokens[at + 2..at + 2 + a]
                    .iter()
                    .all(|v| (1..=2).contains(v)),
                "{text}"
            );
            at += 2 + a;
        }
        assert_eq!(at + 2, tokens.len(), "{text}");
        let spec = InputSpec::from_source(commands).unwrap();
        let text = generate(&spec, &mut Rng::new(seed), Size::Small(5))
            .unwrap()
            .render();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.last(), Some(&"2"), "{text}");
        assert!(
            lines[..lines.len() - 1].iter().all(|l| l.starts_with("1 ")),
            "{text}"
        );
    }
}
