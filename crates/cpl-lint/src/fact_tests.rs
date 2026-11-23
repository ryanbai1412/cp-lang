//! The `Proven*` facts the CLI, the editor and L0414 read: the contract a
//! fact table in `cpl-check` has to meet before it replaces `rules/proven`.

#[test]
fn division_remainder_and_power_follow_the_interpreter() {
    let src = "let a: -7..=9 = read()\nlet b: 2..=4 = read()\nlet q = a / b\nlet r = a % b\nlet s = a ** 2\nlet c = a ** 3\nlet z = a / (b - 2)\n";
    let range = |name: &str| {
        let at = src.find(&format!("let {name} ")).unwrap() as u32 + 4;
        crate::proven_ranges(src)
            .into_iter()
            .find(|r| r.0 == at)
            .map(|r| (r.1, r.2))
    };
    // Euclidean, as `cpl run` computes it: -7 / 2 is -4 and -7 % 4 is 1.
    assert_eq!(range("q"), Some((-4, 4)));
    assert_eq!(range("r"), Some((0, 3)));
    assert_eq!(range("s"), Some((0, 81)));
    assert_eq!(range("c"), Some((-343, 729)));
    assert_eq!(range("z"), None, "the divisor can be 0");
}

fn range_of(src: &str, decl: &str) -> Option<(i128, i128)> {
    let at = src.find(decl).unwrap() as u32 + decl.find(' ').unwrap() as u32 + 1;
    crate::proven_ranges(src)
        .into_iter()
        .find(|r| r.0 == at)
        .map(|r| (r.1, r.2))
}

#[test]
fn a_var_is_in_the_hull_every_assignment_keeps() {
    let head = "let n: 1..=100 = read()\nlet a: 0..=9 = read()\n";
    let cases = [
        (
            "var best = 0\nfor i in 0..n:\n    best max= a * 2\nprint(best)\n",
            Some((0, 18)),
        ),
        (
            "var s = 5\ns = min(s, a)\ns = max(s, abs(a - 20))\nprint(s)\n",
            Some((0, 20)),
        ),
        (
            "var p = n\nif a > 3:\n    p = p % 7\nprint(p)\n",
            Some((0, 100)),
        ),
        (
            "var c = 0\nwhile c < n:\n    c += 1\nprint(c)\n",
            Some((0, 100)),
        ),
        (
            "var c = 0\nwhile c < n:\n    print(c)\n    c += 1\n    c += 1\nprint(c)\n",
            None,
        ),
        ("var c = 0\nfor i in 0..n:\n    c += 1\nprint(c)\n", None),
        (
            "var j = 0\nwhile j < n and a > 2:\n    j = j + 1\nprint(j)\n",
            Some((0, 100)),
        ),
        ("var t = 1\nfor i in 0..n:\n    t = t * 2\nprint(t)\n", None),
        ("var x = 0\nfor i in 0..n:\n    x = i\nprint(x)\n", None),
        (
            "var x = 0\nfn bump():\n    x = 50\nbump()\nprint(x)\n",
            Some((0, 50)),
        ),
        (
            "var x = 0\nvar y = 1\n(x, y) = (y, x + y)\nprint(x)\n",
            None,
        ),
        ("var x = 0\nlet x = 5\nprint(x)\n", None),
    ];
    for (body, want) in cases {
        let decl = &body[..body.find(" =").unwrap()];
        assert_eq!(range_of(&format!("{head}{body}"), decl), want, "{body}");
    }
    let shadowed =
        format!("{head}fn max(a: Int, b: Int) -> Int = a + b\nlet m = max(a, 3)\nprint(m)\n");
    assert_eq!(range_of(&shadowed, "let m"), None, "a user fn named max");
}

#[test]
fn an_if_condition_narrows_the_names_it_compares() {
    let head = "let n: 1..=100 = read()\nlet a: 0..=9 = read()\n";
    let cases = [
        (
            "if 3 <= a < n:\n    let x = a + 0\n    print(x)\n",
            Some((3, 9)),
        ),
        (
            "if n > 50 and a == 2:\n    let x = n * a\n    print(x)\n",
            Some((102, 200)),
        ),
        (
            "if n < 10:\n    let x = n\n    print(x)\nelse:\n    let y = n\n    print(y)\n",
            Some((1, 9)),
        ),
        ("if a >= 20:\n    let x = a\n    print(x)\n", None),
    ];
    for (body, want) in cases {
        assert_eq!(range_of(&format!("{head}{body}"), "let x"), want, "{body}");
    }
    let otherwise = [
        ("if n < 10:\n    print(n)\nelse:\n    let y = n\n    print(y)\n", Some((10, 100))),
        ("if n < 10:\n    print(n)\nelif a != 4:\n    print(a)\nelse:\n    let y = n * a\n    print(y)\n", Some((40, 400))),
        ("if not (n > 30 or a < 2):\n    let y = n + a\n    print(y)\n", Some((3, 39))),
        ("if a < 2 and n < 5:\n    print(n)\nelse:\n    let y = n\n    print(y)\n", Some((1, 100))),
        ("if 2 < a < 5:\n    print(a)\nelse:\n    let y = a\n    print(y)\n", Some((0, 9))),
        ("var i = 0\nwhile a < 5 and i < n:\n    let y = a\n    print(y)\n    i += 1\n", Some((0, 4))),
    ];
    for (body, want) in otherwise {
        assert_eq!(range_of(&format!("{head}{body}"), "let y"), want, "{body}");
    }
}

#[test]
fn a_comparison_of_two_names_bounds_their_difference() {
    let head = "let n: 1..=100 = read()\nlet a: 0..=9 = read()\n";
    let cases = [
        ("if a < n:\n    let x = n - a\n    print(x)\n", Some((1, 100))),
        ("if a >= n:\n    let x = a - n\n    print(x)\n", Some((0, 8))),
        ("if n <= a:\n    print(n)\nelse:\n    let x = a - n\n    print(x)\n", Some((-100, -1))),
        ("if a == n:\n    let x = (n) - (a)\n    print(x)\n", Some((0, 0))),
        ("if a < n:\n    a = 5\n    let x = n - a\n    print(x)\n", Some((-8, 100))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    let mid = (lo + hi) / 2\n    if a < 5:\n        lo = mid + 1\n    else:\n        hi = mid\nlet x = lo - hi\nprint(x)\n", Some((0, 0))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    lo = hi + 5\nlet x = lo - hi\nprint(x)\n", Some((0, 5))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    if a < 5:\n        lo = hi + 1\n        continue\n    lo = hi\nlet x = lo - hi\nprint(x)\n", Some((0, 100))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    lo += 1\n    if lo == 3:\n        break\nlet x = lo - hi\nprint(x)\n", Some((-100, 99))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    let x = hi - lo\n    print(x)\n    lo += 1\n", Some((1, 100))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    lo += 1\n    let x = hi - lo\n    print(x)\n", Some((0, 99))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    let mid = (lo + hi) / 2\n    let x = hi - mid\n    print(x)\n    if a < 5:\n        lo = mid + 1\n    else:\n        hi = mid\n", Some((1, 50))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    let mid = (lo + hi) / 2\n    if a < 5:\n        lo = mid + 1\n    let x = mid - lo\n    print(x)\n", Some((-1, 50))),
    ];
    for (body, want) in cases {
        let src = format!("{head}{body}").replace(
            "let a:",
            if body.contains("a = 5") {
                "var a:"
            } else {
                "let a:"
            },
        );
        assert_eq!(range_of(&src, "let x"), want, "{body}");
    }
}

#[test]
fn a_tuple_read_bounds_each_name_it_binds() {
    let loops = |src: &str| -> Vec<(u128, Option<u128>)> {
        crate::proven_loops(src)
            .into_iter()
            .map(|l| (l.times, l.total))
            .collect()
    };
    let body = "domain V = 1..=n\nfor v in V:\n    for i in 0..q:\n        print(v, i)\n";
    let typed = format!("let (n, q): (1..=200000, 1..=1000) = read()\n{body}");
    assert_eq!(
        loops(&typed),
        [(200000, Some(200000)), (1000, Some(200000000))]
    );
    assert_eq!(crate::domain_sizes(&typed), [("V".to_string(), 200000)]);
    let turbofish = format!("let (n, q) = read<(1..=200000, 1..=1000)>()\n{body}");
    assert_eq!(loops(&turbofish), loops(&typed));
    let unbounded = format!("let (n, q): (Int, 1..=1000) = read()\n{body}");
    assert_eq!(loops(&unbounded), [(1000, None)]);
}

#[test]
fn a_capitalized_let_bounds_loops_like_a_const() {
    let loops = |src: &str| -> Vec<(u128, Option<u128>, String)> {
        crate::proven_loops(src)
            .into_iter()
            .map(|l| (l.times, l.total, l.count))
            .collect()
    };
    let body =
        "for k in 1..=B:\n    print(k)\nfn f():\n    for j in 0..B:\n        print(j)\nf()\n";
    let want = [(7, Some(7), "B".to_string()), (7, Some(7), "B".to_string())];
    assert_eq!(loops(&format!("const B = 7\n{body}")), want);
    assert_eq!(loops(&format!("let B = 7\n{body}")), want);
    assert_eq!(loops(&format!("let B: 1..=7 = read()\n{body}")), want);
    assert_eq!(loops(&format!("let B = 3 + 4\n{body}")), want);
}

#[test]
fn a_for_loop_knows_how_often_it_runs() {
    let loops = |src: &str| -> Vec<(u128, Option<u128>)> {
        crate::proven_loops(src)
            .into_iter()
            .map(|l| (l.times, l.total))
            .collect()
    };
    let head = "let n: 1..=200000 = read()\nlet q: 1..=1000 = read()\ndomain V = 1..=n\n";
    let nested = format!("{head}for v in V:\n    for i in 0..q:\n        print(v, i)\n");
    assert_eq!(
        loops(&nested),
        [(200000, Some(200000)), (1000, Some(200000000))]
    );
    let in_fn = format!(
        "{head}fn f(k: 1..=50):\n    for i in 0..=k:\n        print(i)\nfor v in V:\n    f(3)\n"
    );
    assert_eq!(loops(&in_fn), [(51, Some(51)), (200000, Some(200000))]);
    let in_while = format!(
        "{head}var t = 0\nwhile t < 5:\n    for i in 1..n:\n        print(i)\n    t += 1\n"
    );
    assert_eq!(loops(&in_while), [(199999, None)]);
    let unknown =
        format!("{head}let xs = [1, 2]\nfor x in xs:\n    for i in 0..q:\n        print(x, i)\n");
    assert_eq!(loops(&unknown), [(1000, None)]);
    let counts = |src: &str| -> Vec<(String, Option<String>)> {
        crate::proven_loops(src)
            .into_iter()
            .map(|l| (l.count, l.total_count))
            .collect()
    };
    let s = |t: &str| t.to_string();
    assert_eq!(
        counts(&nested),
        [(s("n"), Some(s("n"))), (s("q"), Some(s("n * q")))]
    );
    let shifted = format!("{head}for v in V:\n    for i in 2..q + 1:\n        for j in 1..=q - 1:\n            print(v, i, j)\n");
    assert_eq!(
        counts(&shifted)[2],
        (s("q - 1"), Some(s("n * 999 * (q - 1)")))
    );
    assert_eq!(counts(&in_fn)[0], (s("51"), Some(s("51"))));
}

#[test]
fn a_fn_knows_how_many_loop_iterations_a_call_makes() {
    let fns = |src: &str| -> Vec<(String, Option<u128>)> {
        let name = |f: &crate::ProvenFn| src[f.name.0 as usize..f.name.1 as usize].to_string();
        crate::proven_fns(src)
            .iter()
            .filter(|f| f.steps != Some(0))
            .map(|f| (name(f), f.steps))
            .collect()
    };
    let head = "let n: 1..=200000 = read()\nlet q: 1..=1000 = read()\ndomain V = 1..=n\n";
    let src = format!(
        "{head}fn grid():\n    for v in V:\n        for i in 0..q:\n            print(v, i)\n    for i in 0..q:\n        print(i)\n\
         fn walk(k: Int):\n    var t = 0\n    while t < k:\n        t += 1\n\
         fn plain(k: Int) -> Int:\n    return k + 1\n\
         fn over(xs: List<Int>):\n    for x in xs:\n        print(x)\n"
    );
    let some = |s: &str, n| (s.to_string(), n);
    assert_eq!(
        fns(&src),
        [
            some("grid", Some(200000 + 200000000 + 1000)),
            some("walk", None),
            some("over", None)
        ]
    );
    let calls = format!(
        "{src}fn twice():\n    for i in 0..2:\n        grid()\n    plain(3)\n\
         fn again(k: Int):\n    again(k - 1)\n    grid()\n\
         fn mapped():\n    let ys = [1, 2].map(x => grid())\n"
    );
    let found = fns(&calls);
    assert_eq!(
        found[3..],
        [
            some("twice", Some(2 + 2 * 200201000)),
            some("again", None),
            some("mapped", None)
        ]
    );
    let counts: Vec<Option<String>> = crate::proven_fns(&calls)
        .into_iter()
        .filter(|f| f.steps != Some(0))
        .map(|f| f.count)
        .collect();
    let s = |t: &str| Some(t.to_string());
    assert_eq!(
        counts,
        [
            s("n + n * q + q"),
            None,
            None,
            s("2 + 2 * n + 2 * n * q + 2 * q"),
            None,
            None
        ]
    );
}

#[test]
fn a_fn_lists_the_calls_its_loop_count_leaves_out() {
    let src = "let n: 1..=10 = read()\nfn inner() -> Int:\n    return 1\n\
               fn outer(xs: List<Int>):\n    for i in 0..n:\n        print(inner(), xs.sorted().len)\n\
               \x20   fn nested():\n        xs.reverse()\n    (inner)()\n    let p = P(1)\n\
               struct P:\n    a: Int\n";
    let callees: Vec<Vec<&str>> = crate::proven_fns(src)
        .iter()
        .map(|f| {
            f.callees
                .iter()
                .map(|&(s, e)| &src[s as usize..e as usize])
                .collect()
        })
        .collect();
    // `inner` is in the loop count; `nested` speaks for its own calls.
    assert_eq!(
        callees,
        [vec![], vec!["reverse"], vec!["print", "sorted", "(inner)"]]
    );
    let builds: Vec<usize> = crate::proven_fns(src)
        .iter()
        .map(|f| f.builds.len())
        .collect();
    assert_eq!(builds, [0, 0, 1]);
}

#[test]
fn the_prover_names_the_inputs_an_overflow_needs() {
    let overflows = |src: &str| -> Vec<Option<String>> {
        crate::proven_expr_ranges(src)
            .into_iter()
            .map(|e| e.overflow)
            .collect()
    };
    let head = "let n: 1..=200000 = read()\nlet a: -1e9..=1e9 = read()\nlet m = n * n\n";
    let some = |s: &str| Some(s.to_string());
    assert_eq!(
        overflows(&format!("{head}print(m * m)\n")),
        [None, some("n = 55109")]
    );
    assert_eq!(
        overflows(&format!("{head}print(a * a * a)\n")),
        [None, None, some("a = 2097152")]
    );
    assert_eq!(
        overflows(&format!("{head}print(n * n * n)\n")),
        [None, None, None]
    );
    let narrowed = format!("{head}if n < 1000:\n    print(n * n * n * n * n)\n");
    assert!(
        overflows(&narrowed).iter().all(Option::is_none),
        "a narrowed name is not an input"
    );
    let var = format!("{head}var t = n\nt = n * n\nprint(t * t)\n");
    assert!(
        overflows(&var).iter().all(Option::is_none),
        "a var is not followed"
    );
}

#[test]
fn vars_that_assign_each_other_are_solved_together() {
    let search = "var lo = 0\nvar hi = n\nwhile lo < hi:\n    let mid = (lo + hi) / 2\n    if a < 5:\n        lo = mid + 1\n    else:\n        hi = mid\nprint(lo, hi)\n";
    let small = format!("let n: 1..=100 = read()\nlet a: 0..=9 = read()\n{search}");
    assert_eq!(range_of(&small, "var lo"), Some((0, 101)));
    assert_eq!(range_of(&small, "var hi"), Some((0, 100)));
    assert_eq!(range_of(&small, "let mid"), Some((0, 100)));
    let wide = small.replace("1..=100", "1..=1e18");
    assert_eq!(
        range_of(&wide, "var lo"),
        Some((0, 1_000_000_000_000_000_001))
    );
    let cases = [
        // `j` grows without bound, so `i`, which copies it, has no range.
        (
            "var i = 0\nvar j = 0\nwhile i < n:\n    j += 1\n    i = j\nprint(i)\n",
            "var i",
            None,
        ),
        // `k` is bound twice in the file, so its value may be either one.
        (
            "let k = 500\nvar x = 0\nfor _ in 0..n:\n    let k = a\n    x = k\nprint(x)\n",
            "var x",
            None,
        ),
        (
            "var x = 0\nvar y = a\nfor _ in 0..n:\n    x = y * 2\n    y = min(y, x)\nprint(x)\n",
            "var x",
            Some((0, 18)),
        ),
    ];
    for (body, decl, want) in cases {
        let src = format!("let n: 1..=100 = read()\nlet a: 0..=9 = read()\n{body}");
        assert_eq!(range_of(&src, decl), want, "{body}");
    }
}

#[test]
fn a_loop_that_exits_leaves_its_condition_false() {
    let head = "let n: 1..=100 = read()\nlet a: 0..=9 = read()\nvar x = n\n";
    let halve = "while x > 9:\n    x = x / 2\n";
    let cases = [
        (format!("{halve}let y = x\nprint(y)\n"), Some((1, 9))),
        (
            format!(
                "for _ in 0..a:\n    {}    let y = x + 0\n    print(y)\n",
                halve.replace("\n    ", "\n        ")
            ),
            Some((1, 9)),
        ),
        (
            "while x > 9:\n    x = x / 2\n    if a == 3:\n        break\nlet y = x\nprint(y)\n"
                .to_string(),
            Some((1, 100)),
        ),
        (
            format!("{halve}x = 50\nlet y = x\nprint(y)\n"),
            Some((1, 100)),
        ),
        (
            format!("fn grow():\n    x = 100\n{halve}grow()\nlet y = x\nprint(y)\n"),
            Some((0, 100)),
        ),
        (
            "fn grow():\n    x = 100\nif x < 10:\n    grow()\n    let y = x\n    print(y)\n"
                .to_string(),
            Some((1, 100)),
        ),
    ];
    for (body, want) in cases {
        assert_eq!(range_of(&format!("{head}{body}"), "let y"), want, "{body}");
    }
}

/// A binary search over a list it never resizes keeps `0 <= lo <= mid <
/// hi <= ys.len`, so `ys[mid]` is in bounds; so is `ys[i]` under
/// `for i in 0..ys.len`. A list that is pushed to, or an index past the
/// end, proves nothing.
#[test]
fn an_index_below_a_fixed_length_is_proven_in_bounds() {
    let search = "fn count(ys: List<Int>, p: Int) -> Int:\n    var (lo, hi) = (0, ys.len)\n    while lo < hi:\n        let mid = (lo + hi) / 2\n        if ys[mid] < p: lo = mid + 1\n        else: hi = mid\n    ys.len - lo\nprint(count([1, 2], 2))\n";
    let proven = |src: &str| -> Vec<(String, String)> {
        crate::proven_indexes(src)
            .into_iter()
            .map(|p| {
                (
                    src[p.access.0 as usize..p.access.1 as usize].to_string(),
                    p.proof,
                )
            })
            .collect()
    };
    let pair = |i: &str, l: &str| (i.to_string(), l.to_string());
    assert_eq!(
        proven(search),
        [pair("ys[mid]", "0 <= lo <= mid < hi <= ys.len")]
    );
    let pushed = search.replace(
        "print(count",
        "fn grow(ys: List<Int>):\n    ys.push(1)\nprint(count",
    );
    assert_eq!(proven(&pushed), []);
    let past = search.replace("(0, ys.len)", "(0, ys.len + 1)");
    assert_eq!(proven(&past), []);
    let each = "fn f(xs: List<Int>) -> Int:\n    var s = 0\n    for i in 0..xs.len:\n        s += xs[i] + xs[i + 1]\n    s\nprint(f([1]))\n";
    assert_eq!(proven(each), [pair("xs[i]", "0 <= i < xs.len")]);
    // No hover or hint shows a range a `.len` bounds only by its type.
    assert!(
        crate::proven_ranges(search).is_empty(),
        "{:?}",
        crate::proven_ranges(search)
    );
}

#[test]
fn a_list_is_indexed_safely_once_nothing_changes_its_length() {
    let proofs = |src: &str| -> Vec<String> {
        crate::proven_indexes(src)
            .into_iter()
            .map(|p| p.proof)
            .collect()
    };
    let built = "fn f(p: Int) -> Int:
    var ys: List<Int> = []
    for i in 0..5:
        ys.push(i)
    var (lo, hi) = (0, ys.len)
    while lo < hi:
        let mid = (lo + hi) / 2
        if ys[mid] < p: lo = mid + 1
        else: hi = mid
    lo
print(f(2))
";
    assert_eq!(proofs(built), ["0 <= lo <= mid < hi <= ys.len"]);
    let popped = built.replace(
        "else: hi = mid",
        "else:\n            hi = mid\n            ys.pop()",
    );
    assert_eq!(proofs(&popped), Vec::<String>::new());
    let by_fn = built.replace("    lo\n", "    shrink()\n    lo\n").replace(
        "    var (lo",
        "    fn shrink():\n        ys.pop()\n    var (lo",
    );
    assert_eq!(proofs(&by_fn), Vec::<String>::new());
    // A change later in a loop around the search reaches it on the next turn.
    let again = "fn f(p: Int) -> Int:
    var ys: List<Int> = [1]
    var lo = 0
    for t in 0..3:
        var hi = ys.len
        while lo < hi:
            let mid = (lo + hi) / 2
            if ys[mid] < p: lo = mid + 1
            else: hi = mid
        ys.pop()
    lo
print(f(2))
";
    assert_eq!(proofs(again), ["0 <= lo <= mid < hi <= ys.len"]);
    let stale = again
        .replace("        var hi = ys.len\n", "")
        .replace("    var lo = 0\n", "    var lo = 0\n    var hi = ys.len\n");
    assert_eq!(proofs(&stale), Vec::<String>::new());
    // A breadth-first queue: each read comes before the pushes after it.
    let queue = "var q: List<Int> = [0]
var head = 0
while head < q.len:
    let u = q[head]
    head += 1
    if u < 9: q.push(u + 1)
print(head)
";
    assert_eq!(proofs(queue), ["0 <= head < q.len"]);
    let late = queue.replace(
        "    let u = q[head]\n    head += 1\n",
        "    q.push(1)\n    let u = q[head]\n    head += 1\n",
    );
    assert_eq!(proofs(&late), Vec::<String>::new());
    let same = queue.replace("    let u = q[head]\n", "    let u = q.pop()! + q[head]\n");
    assert_eq!(proofs(&same), Vec::<String>::new());
    // Reading the queue between the test and the index changes nothing.
    let read = queue.replace(
        "    let u = q[head]\n",
        "    let seen = q.contains(9)\n    let u = q[head]\n",
    );
    assert_eq!(proofs(&read), ["0 <= head < q.len"]);
    // Reordering in place keeps the length; emptying it does not.
    let sorted = queue.replace(
        "    let u = q[head]\n",
        "    q.sort()\n    q.reverse()\n    let u = q[head]\n",
    );
    assert_eq!(proofs(&sorted), ["0 <= head < q.len"]);
    let cleared = queue.replace(
        "    let u = q[head]\n",
        "    q.clear()\n    let u = q[head]\n",
    );
    assert_eq!(proofs(&cleared), Vec::<String>::new());
    assert!([&late, &same]
        .iter()
        .all(|src| cpl_syntax::parse_file(src).diagnostics.is_empty()));
    let port = "var routes: List<Int> = []\nfor _ in 0..3:\n    routes.push(read())\nvar s = 0\nfor i in 0..routes.len:\n    s += routes[i]\n    s += routes[i]\nprint(s)\n";
    let routes: Vec<String> = crate::proven_indexes(port)
        .into_iter()
        .map(|p| {
            format!(
                "{} {}",
                &port[p.access.0 as usize..p.access.1 as usize],
                p.proof
            )
        })
        .collect();
    assert_eq!(
        routes[..2],
        [
            "routes[i] 0 <= i < routes.len",
            "routes[i] 0 <= i < routes.len"
        ]
    );
}

#[test]
fn a_row_is_indexed_safely_below_its_own_length() {
    let proofs = |src: &str| -> Vec<String> {
        crate::proven_indexes(src)
            .into_iter()
            .map(|p| {
                format!(
                    "{} {}",
                    &src[p.access.0 as usize..p.access.1 as usize],
                    p.proof
                )
            })
            .collect()
    };
    let walk = "fn f(g: List<List<Int>>) -> Int:
    var s = 0
    for u in 0..g.len:
        for j in 0..g[u].len:
            s += g[u][j]
    s
print(f([[1], [2, 3]]))
";
    let rows = [
        "g[u] 0 <= u < g.len",
        "g[u] 0 <= u < g.len",
        "g[u][j] 0 <= j < g[u].len",
    ];
    assert_eq!(proofs(walk), rows);
    let grown = walk.replace(
        "            s += g[u][j]",
        "            s += g[u][j]\n            g[u].push(1)",
    );
    assert_eq!(proofs(&grown), ["g[u] 0 <= u < g.len"; 3]);
    let other = walk.replace("s += g[u][j]", "s += g[u][j]\n            u = 0");
    assert_eq!(proofs(&other), ["g[u] 0 <= u < g.len"]);
    let cell = walk.replace("s += g[u][j]", "g[u][j] = s");
    assert_eq!(proofs(&cell), rows);
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../corpus/problems/ioi12p6/port.cp"
    );
    let port = &std::fs::read_to_string(path).unwrap();
    assert!(
        proofs(port).contains(&"kids[v][i] 0 <= i < kids[v].len".to_string()),
        "{:?}",
        proofs(port)
    );
}

#[test]
fn a_method_call_knows_the_size_of_an_array_receiver() {
    let sizes = |src: &str| -> Vec<(String, u128, String)> {
        crate::proven_sizes(src)
            .into_iter()
            .map(|s| {
                (
                    src[s.receiver.0 as usize..s.receiver.1 as usize].to_string(),
                    s.max,
                    s.count,
                )
            })
            .collect()
    };
    let head =
        "let n: 1..=200000 = read()\nlet q: 1..=1000 = read()\ndomain V = 1..=n\ndomain R = 0..3\n";
    let src = format!(
        "{head}let a: [0..q]Int = read()\nfn f(b: [V, R]Int, c: List<Int>):\n    print(a.sorted(), b.sorted(), c.sorted())\n\
         fn g(a: List<Int>):\n    print(a.sorted())\n"
    );
    let s = |t: &str| t.to_string();
    assert_eq!(
        sizes(&src),
        [(s("a"), 1000, s("q")), (s("b"), 600000, s("n * 3"))]
    );
}

#[test]
fn a_list_only_pushed_in_counted_loops_has_a_bounded_size() {
    let sizes = |src: &str| -> Vec<(String, u128, String)> {
        crate::proven_sizes(src)
            .into_iter()
            .map(|s| {
                (
                    src[s.receiver.0 as usize..s.receiver.1 as usize].to_string(),
                    s.max,
                    s.count,
                )
            })
            .collect()
    };
    let head = "let n: 1..=200000 = read()\nlet q: 1..=1000 = read()\ndomain V = 1..=n\n";
    let src = format!(
        "{head}fn f():\n    var xs: List<Int> = []\n    for v in V:\n        xs.push(v)\n        for j in 0..q:\n            xs.push(j)\n    xs.push(0)\n    xs.pop()\n    print(xs.sorted())\n\
         fn g():\n    var ys: List<Int> = []\n    var i = 0\n    while i < n:\n        ys.push(i)\n        i += 1\n    print(ys.sorted())\n\
         fn h():\n    var zs: List<Int> = []\n    zs.push(1)\n    zs.extend([1, 2])\n    print(zs.sorted())\n\
         fn k():\n    var ws: List<Int> = []\n    let add = (x: Int) => ws.push(x)\n    add(1)\n    print(ws.sorted())\n"
    );
    let s = |t: &str| t.to_string();
    let got = sizes(&src);
    assert_eq!(
        got.iter().rfind(|(r, ..)| r == "xs"),
        Some(&(s("xs"), 200200001, s("n + n * q + 1")))
    );
    assert!(got.iter().all(|(r, ..)| r == "xs"), "{got:?}");
}

#[test]
fn a_bounded_list_passed_to_print_has_its_size() {
    let sizes = |src: &str| -> Vec<(String, u128, String)> {
        crate::proven_sizes(src)
            .into_iter()
            .map(|s| {
                (
                    src[s.receiver.0 as usize..s.receiver.1 as usize].to_string(),
                    s.max,
                    s.count,
                )
            })
            .collect()
    };
    let src = "let n: 1..=200000 = read()\nfn f():\n    var xs: List<Int> = []\n    for i in 0..n:\n        xs.push(i)\n    print(\"!\", xs)\n    let r: Int = ask(xs)\n    print(r)\n";
    let s = |t: &str| t.to_string();
    let at = |r: &str| {
        sizes(src)
            .into_iter()
            .filter(|(x, ..)| x == r)
            .collect::<Vec<_>>()
    };
    assert_eq!(at("xs"), vec![(s("xs"), 200000, s("n")); 3]);
    let src = "let k: 1..=7 = read()\nfn g():\n    var got: List<Int> = []\n    for i in 0..k:\n        got.push(i * i)\n        print(i)\n    print(\"!\", got)\n";
    assert_eq!(sizes(src), vec![(s("got"), 7, s("k")); 2]);
}

#[test]
fn parsed_entry_points_match_source_entry_points_and_isolate_local_fn_shadowing() {
    let src = "let n: 1..=200000 = read()\nlet m = n * n\nfn helper(x: 1..=10) -> Int:\n    fn min(a: Int, b: Int) -> Int: a + b\n    let y = min(x, 5)\n    y\nlet ok = min(n, 10)\nprint(m * m, helper(1), ok)\n";
    let parsed = cpl_syntax::parse_file(src);
    assert!(parsed.diagnostics.is_empty());
    let items: Vec<(u32, &cpl_syntax::ParsedItem)> =
        parsed.items.iter().map(|(o, p)| (*o, p)).collect();
    assert_eq!(
        crate::lint_parsed(src, &items, &crate::Config::default()),
        crate::lint(src, &crate::Config::default()),
    );
    assert_eq!(
        crate::proven_expr_ranges_parsed(src, &items),
        crate::proven_expr_ranges(src),
    );
    // `helper`'s local `fn min` shadows prelude `min` inside `helper`, not at top level.
    assert_eq!(range_of(src, "let y"), None);
    assert_eq!(range_of(src, "let ok"), Some((1, 10)));
}
