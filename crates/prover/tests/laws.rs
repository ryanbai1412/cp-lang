//! The structures of `stdlib/algebra.cp` and `stdlib/lazy.cp`, checked
//! with the laws as they are written there, and the ways an instance
//! goes wrong.

use cpl_prover::{Context, Report, Structure, Verdict};

fn proved(report: &Report) {
    assert!(report.holds(), "{report}");
}

const INT: [(&str, &str); 1] = [("T", "Int")];

// ---- the constants of stdlib/algebra.cp -----------------------------------

#[test]
fn sum_is_a_commutative_group() {
    // pub const Sum<T> = Group<T>(id: T.zero, op: (a, b) => a + b, inv: a => -a)
    let fields = [("id", "0"), ("op", "(a, b) => a + b"), ("inv", "a => -a")];
    let report = Structure::group()
        .commutative()
        .check(&INT, &fields)
        .unwrap();
    proved(&report);
    assert_eq!(
        report.to_string(),
        "Group keeps law assoc\nGroup keeps law ident\nGroup keeps law inverse\nGroup keeps law commutes"
    );
}

#[test]
fn prod_is_a_monoid_and_not_a_group() {
    // pub const Prod<T> = Monoid<T>(id: T.one, op: (a, b) => a * b)
    let fields = [("id", "1"), ("op", "(a, b) => a * b")];
    proved(
        &Structure::monoid()
            .commutative()
            .check(&INT, &fields)
            .unwrap(),
    );
    // No integer inverse: there is no `inv` with a * inv(a) == 1.
    let fields = [("id", "1"), ("op", "(a, b) => a * b"), ("inv", "a => a")];
    let report = Structure::group().check(&INT, &fields).unwrap();
    assert!(report.verdict("assoc").is_proved() && report.verdict("ident").is_proved());
    let broken = report.law("inverse").unwrap();
    assert_eq!(broken.verdict.to_string(), "fails for a = 0");
}

#[test]
fn max_and_min_are_idempotent_monoids() {
    // pub const Max<T> = Monoid<T>(id: T.MIN, op: (a, b) => max(a, b))
    let max = [("id", "Int.MIN"), ("op", "(a, b) => max(a, b)")];
    proved(
        &Structure::monoid()
            .commutative()
            .idempotent()
            .check(&INT, &max)
            .unwrap(),
    );
    // pub const Min<T> = Monoid<T>(id: T.MAX, op: (a, b) => min(a, b))
    let min = [("id", "Int.MAX"), ("op", "(a, b) => min(a, b)")];
    proved(
        &Structure::monoid()
            .commutative()
            .idempotent()
            .check(&INT, &min)
            .unwrap(),
    );
}

#[test]
fn the_wrong_identity_for_max_is_caught() {
    // 0 is the identity of max only for non-negative values.
    let fields = [("id", "0"), ("op", "(a, b) => max(a, b)")];
    let report = Structure::monoid().check(&INT, &fields).unwrap();
    assert_eq!(
        report.to_string(),
        "Monoid keeps law assoc\nMonoid breaks law ident\n  a = -1\n  op(id, a) = 0   a = -1"
    );
    // With a type that has no negative values the same instance is fine.
    proved(
        &Structure::monoid()
            .check(&[("T", "0..=1e9")], &fields)
            .unwrap(),
    );
}

#[test]
fn xor_is_a_group_that_is_its_own_inverse() {
    // pub const Xor<T> = Group<T>(id: T.zero, op: (a, b) => a ^ b, inv: a => a)
    let fields = [("id", "0"), ("op", "(a, b) => a ^ b"), ("inv", "a => a")];
    proved(
        &Structure::group()
            .commutative()
            .check(&INT, &fields)
            .unwrap(),
    );
}

#[test]
fn bit_and_gcd_monoids() {
    let and = [("id", "-1"), ("op", "(a, b) => a & b")];
    proved(
        &Structure::monoid()
            .commutative()
            .idempotent()
            .check(&INT, &and)
            .unwrap(),
    );
    let or = [("id", "0"), ("op", "(a, b) => a | b")];
    proved(
        &Structure::monoid()
            .commutative()
            .idempotent()
            .check(&INT, &or)
            .unwrap(),
    );
    // gcd(0, a) is |a|: the identity law needs non-negative values.
    let gcd = [("id", "0"), ("op", "(a, b) => gcd(a, b)")];
    let report = Structure::monoid().check(&INT, &gcd).unwrap();
    assert!(report.verdict("assoc").is_proved());
    assert_eq!(report.verdict("ident").to_string(), "fails for a = -1");
    proved(
        &Structure::monoid()
            .commutative()
            .check(&[("T", "0..=1e18")], &gcd)
            .unwrap(),
    );
}

#[test]
fn less_and_greater_are_strict_orders() {
    // pub const Less<T> = Order<T>(lt: (a, b) => a < b); Greater flips it.
    proved(
        &Structure::order()
            .check(&[("T", "Int")], &[("lt", "(a, b) => a < b")])
            .unwrap(),
    );
    proved(
        &Structure::order()
            .check(&[("T", "Int")], &[("lt", "(a, b) => b < a")])
            .unwrap(),
    );
    // Lexicographic on pairs.
    let lex = [("lt", "(a, b) => a.0 < b.0 or (a.0 == b.0 and a.1 < b.1)")];
    proved(
        &Structure::order()
            .check(&[("T", "(Int, Int)")], &lex)
            .unwrap(),
    );
}

#[test]
fn a_comparison_that_is_not_strict_is_not_an_order() {
    // The classic sort comparator bug: `<=` is not irreflexive.
    let report = Structure::order()
        .check(&[("T", "Int")], &[("lt", "(a, b) => a <= b")])
        .unwrap();
    assert_eq!(report.verdict("irreflexive").to_string(), "fails for a = 0");
    assert!(report.verdict("transitive").is_proved());
    // "Close enough" is not transitive.
    let near = Structure::order()
        .check(
            &[("T", "Int")],
            &[("lt", "(a, b) => a + 1 < b or b + 1 < a")],
        )
        .unwrap();
    assert!(near.verdict("irreflexive").is_proved());
    assert_eq!(
        near.verdict("transitive").to_string(),
        "fails for a = 0, b = 2, c = 0"
    );
}

// ---- derived monoids --------------------------------------------------------

#[test]
fn opt_lifts_a_semigroup_to_a_monoid() {
    // pub fn opt<T>(s: Semigroup<T>) -> Monoid<?T>: none is the identity.
    let fields = [
        ("id", "none"),
        (
            "op",
            "(a, b) => if a == none: b else: if b == none: a else: min(a!, b!)",
        ),
    ];
    proved(
        &Structure::monoid()
            .commutative()
            .check(&[("T", "?Int")], &fields)
            .unwrap(),
    );
    // Dropping the right operand when the left is absent is not a monoid.
    let lossy = [
        ("id", "none"),
        (
            "op",
            "(a, b) => if a == none: none else: if b == none: a else: min(a!, b!)",
        ),
    ];
    let report = Structure::monoid().check(&[("T", "?Int")], &lossy).unwrap();
    assert_eq!(report.verdict("ident").to_string(), "fails for a = 0");
}

#[test]
fn both_pairs_two_monoids() {
    // pub fn both<A, B>(x: Monoid<A>, y: Monoid<B>) -> Monoid<(A, B)>; SumLen is one.
    let sum_len = [("id", "(0, 0)"), ("op", "(a, b) => (a.0 + b.0, a.1 + b.1)")];
    proved(
        &Structure::monoid()
            .commutative()
            .check(&[("T", "(Int, Int)")], &sum_len)
            .unwrap(),
    );
    let max_min = [
        ("id", "(Int.MIN, Int.MAX)"),
        ("op", "(a, b) => (max(a.0, b.0), min(a.1, b.1))"),
    ];
    proved(
        &Structure::monoid()
            .check(&[("T", "(Int, Int)")], &max_min)
            .unwrap(),
    );
}

#[test]
fn flip_keeps_the_laws_and_loses_nothing() {
    // pub fn flip<T>(m: Monoid<T>) -> Monoid<T> = Monoid(m.id, (a, b) => m.op(b, a))
    let mut ctx = Context::new();
    ctx.define("first", "(a, b) => if a == none: b else: a")
        .unwrap();
    let fields = [("id", "none"), ("op", "(a, b) => first(b, a)")];
    let report = Structure::monoid()
        .check_in(&ctx, &[("T", "?Int")], &fields)
        .unwrap();
    proved(&report);
    // "Keep the first" is not commutative.
    let commutes = Structure::semigroup()
        .commutative()
        .check_in(&ctx, &[("T", "?Int")], &[("op", "first")])
        .unwrap();
    assert_eq!(
        commutes.verdict("commutes").to_string(),
        "fails for a = 1, b = 0"
    );
}

// ---- segment-tree monoids ---------------------------------------------------

#[test]
fn the_count_of_the_maximum() {
    // stdlib/tour.cp: (best, how many times it occurs).
    let fields = [
        ("id", "(Int.MIN, 0)"),
        (
            "op",
            "(x, y) => if x.0 > y.0: x else: if y.0 > x.0: y else: (x.0, x.1 + y.1)",
        ),
    ];
    proved(
        &Structure::monoid()
            .commutative()
            .check(&[("T", "(Int, Int)")], &fields)
            .unwrap(),
    );
    // Forgetting to add the counts on a tie breaks commutativity, not associativity.
    let ties = [
        ("id", "(Int.MIN, 0)"),
        ("op", "(x, y) => if x.0 >= y.0: x else: y"),
    ];
    let report = Structure::monoid()
        .commutative()
        .check(&[("T", "(Int, Int)")], &ties)
        .unwrap();
    assert!(report.verdict("assoc").is_proved());
    assert_eq!(
        report.verdict("commutes").to_string(),
        "fails for a = (0, 1), b = (0, 0)"
    );
}

#[test]
fn maximum_prefix_sum() {
    let ty = [("T", "(sum: Int, pre: Int)")];
    // The right one: the prefix of the right half starts after the whole left half.
    let good = [
        ("id", "(sum: 0, pre: Int.MIN)"),
        (
            "op",
            "(x, y) => (sum: x.sum + y.sum, pre: max(x.pre, x.sum + y.pre))",
        ),
    ];
    assert!(Structure::semigroup()
        .check(&ty, &good[1..])
        .unwrap()
        .holds());
    // Adding the right half's own sum instead of the left one is the usual slip.
    let slip = [(
        "op",
        "(x, y) => (sum: x.sum + y.sum, pre: max(x.pre, y.sum + y.pre))",
    )];
    let report = Structure::semigroup().check(&ty, &slip).unwrap();
    assert_eq!(
        report.to_string(),
        "Semigroup breaks law assoc\n  a = (sum: 0, pre: 0), b = (sum: 0, pre: 0), c = (sum: 1, pre: 0)\n  \
         op(op(a, b), c) = (sum: 1, pre: 1)   op(a, op(b, c)) = (sum: 1, pre: 2)"
    );
    // Forgetting the left sum altogether is still associative: it is the
    // pair (sum, max), a perfectly good monoid that answers another question.
    let pair = [(
        "op",
        "(x, y) => (sum: x.sum + y.sum, pre: max(x.pre, y.pre))",
    )];
    assert!(Structure::semigroup().check(&ty, &pair).unwrap().holds());
}

#[test]
fn maximum_subarray_sum() {
    let ty = [("T", "(sum: Int, pre: Int, suf: Int, best: Int)")];
    let op = "(x, y) => (sum: x.sum + y.sum, pre: max(x.pre, x.sum + y.pre), \
              suf: max(y.suf, y.sum + x.suf), best: max(x.best, y.best, x.suf + y.pre))";
    proved(&Structure::semigroup().check(&ty, &[("op", op)]).unwrap());
    // Joining at the wrong ends: prefix of the left with suffix of the right.
    let wrong = "(x, y) => (sum: x.sum + y.sum, pre: max(x.pre, x.sum + y.pre), \
                 suf: max(y.suf, y.sum + x.suf), best: max(x.best, y.best, x.pre + y.suf))";
    let report = Structure::semigroup().check(&ty, &[("op", wrong)]).unwrap();
    let Verdict::Refuted(counterexample) = report.verdict("assoc") else {
        panic!("{report}");
    };
    assert_eq!(counterexample.bindings.len(), 3);
    assert_eq!(
        report.law("assoc").unwrap().broken.as_ref().unwrap().lhs,
        "op(op(a, b), c)"
    );
}

// ---- modular arithmetic -------------------------------------------------------

#[test]
fn arithmetic_modulo_a_constant() {
    let ty = [("T", "0..998244353")];
    let add = [
        ("id", "0"),
        ("op", "(a, b) => (a + b) % 998244353"),
        ("inv", "a => (998244353 - a) % 998244353"),
    ];
    let closed = "law closed(a: T, b: T) = op(a, b) in 0..998244353";
    proved(
        &Structure::group()
            .commutative()
            .with_law(closed)
            .unwrap()
            .check(&ty, &add)
            .unwrap(),
    );
    let mul = [("id", "1"), ("op", "(a, b) => a * b % 998244353")];
    proved(
        &Structure::monoid()
            .commutative()
            .with_law(closed)
            .unwrap()
            .check(&ty, &mul)
            .unwrap(),
    );
    // Forgetting to reduce breaks closure, and the report says where.
    let raw = [("id", "0"), ("op", "(a, b) => a + b")];
    let report = Structure::monoid()
        .with_law(closed)
        .unwrap()
        .check(&ty, &raw)
        .unwrap();
    assert_eq!(
        report.verdict("closed").to_string(),
        "fails for a = 998244352, b = 1"
    );
}

#[test]
fn arithmetic_modulo_a_runtime_modulus() {
    // ModR: the modulus is read from the input.
    let mut ctx = Context::new();
    ctx.var("m", "1..=1e9").unwrap();
    let ty = [("T", "0..m")];
    let closed = "law closed(a: T, b: T) = op(a, b) in 0..m";
    let add = [("id", "0"), ("op", "(a, b) => (a + b) % m")];
    proved(
        &Structure::monoid()
            .commutative()
            .with_law(closed)
            .unwrap()
            .check_in(&ctx, &ty, &add)
            .unwrap(),
    );
    let mul = [("id", "1 % m"), ("op", "(a, b) => a * b % m")];
    proved(
        &Structure::monoid()
            .commutative()
            .with_law(closed)
            .unwrap()
            .check_in(&ctx, &ty, &mul)
            .unwrap(),
    );
    // `id: 1` still satisfies the identity law when m = 1 (1 * 0 % 1 == 0),
    // but it is not a value of `0..m` there, which is its own obligation.
    let one = [("id", "1"), ("op", "(a, b) => a * b % m")];
    let report = Structure::monoid().check_in(&ctx, &ty, &one).unwrap();
    assert!(report.verdict("ident").is_proved(), "{report}");
    assert_eq!(
        ctx.clone().check_index("1", "0..m").unwrap().to_string(),
        "fails for m = 1"
    );
}

// ---- actions (stdlib/lazy.cp) -----------------------------------------------------

#[test]
fn add_on_max_is_an_action() {
    // MaxOpt = Monoid<?Int>(id: none, op: (a, b) => if a == none: b else: if b == none: a else: max(a!, b!))
    // AddMax = Action<?Int, Int>(id: 0, compose: (f, g) => f + g,
    //                            apply: (f, x) => if x == none: none else: x! + f)
    let types = [("T", "?Int"), ("F", "Int")];
    let fields = [
        ("id", "0"),
        ("compose", "(f, g) => f + g"),
        ("apply", "(f, x) => if x == none: none else: x! + f"),
        (
            "op",
            "(a, b) => if a == none: b else: if b == none: a else: max(a!, b!)",
        ),
    ];
    proved(
        &Structure::action()
            .distributive()
            .check(&types, &fields)
            .unwrap(),
    );
    let max_opt = [("id", "none"), fields[3]];
    proved(
        &Structure::monoid()
            .commutative()
            .idempotent()
            .check(&types[..1], &max_opt)
            .unwrap(),
    );
}

#[test]
fn an_empty_marker_inside_int_breaks_composition() {
    // What AddMax was before: Int.MIN as "empty", over Max<Int>. A sum can
    // land exactly on the marker, and then the next add is dropped.
    let types = [("T", "Int"), ("F", "Int")];
    let fields = [
        ("id", "0"),
        ("compose", "(f, g) => f + g"),
        ("apply", "(f, x) => if x == Int.MIN: x else: x + f"),
        ("op", "(a, b) => max(a, b)"),
    ];
    let report = Structure::action()
        .distributive()
        .check(&types, &fields)
        .unwrap();
    assert!(report.verdict("ident").is_proved(), "{report}");
    assert!(report.verdict("distributes").is_proved(), "{report}");
    assert_eq!(
        report.verdict("composes").to_string(),
        "fails for f = 1, g = -1, a = -9223372036854775806"
    );
    // Clamping the sum above the marker does not repair it: the clamp
    // forgets how far below the sum went.
    let clamped = [
        fields[0],
        fields[1],
        (
            "apply",
            "(f, x) => if x == Int.MIN: x else: max(Int.MIN + 1, x + f)",
        ),
        fields[3],
    ];
    let report = Structure::action().check(&types, &clamped).unwrap();
    assert!(!report.verdict("composes").is_proved(), "{report}");
}

#[test]
fn add_on_sum_needs_the_length() {
    // AddSum = Action<(Int, Int), Int>(id: 0, compose: (f, g) => f + g,
    //                                  apply: (f, x) => (x.0 + f * x.1, x.1))
    let types = [("T", "(Int, Int)"), ("F", "Int")];
    let fields = [
        ("id", "0"),
        ("compose", "(f, g) => f + g"),
        ("apply", "(f, x) => (x.0 + f * x.1, x.1)"),
        ("op", "(a, b) => (a.0 + b.0, a.1 + b.1)"),
    ];
    proved(
        &Structure::action()
            .distributive()
            .check(&types, &fields)
            .unwrap(),
    );
    // Without the length the update does not distribute over the merge.
    let no_len = [
        ("id", "0"),
        ("compose", "(f, g) => f + g"),
        ("apply", "(f, x) => x + f"),
        ("op", "(a, b) => a + b"),
    ];
    let report = Structure::action()
        .distributive()
        .check(&[("T", "Int"), ("F", "Int")], &no_len)
        .unwrap();
    assert!(report.verdict("ident").is_proved() && report.verdict("composes").is_proved());
    assert_eq!(
        report
            .law("distributes")
            .map(|law| report_line(&report, &law.law.name))
            .unwrap(),
        "f = 1, a = 0, b = 0 | apply(f, op(a, b)) = 1 | op(apply(f, a), apply(f, b)) = 2"
    );
}

fn report_line(report: &Report, law: &str) -> String {
    let result = report.law(law).unwrap();
    let broken = result.broken.as_ref().unwrap();
    let counterexample = result.verdict.counterexample().unwrap();
    format!(
        "{counterexample} | {} = {} | {} = {}",
        broken.lhs, broken.left, broken.rhs, broken.right
    )
}

#[test]
fn affine_maps_compose_in_one_order_only() {
    // f = (mul, add) acts on (sum, len): the range "multiply then add" update.
    let types = [("T", "(sum: Int, len: Int)"), ("F", "(Int, Int)")];
    let apply = (
        "apply",
        "(f, x) => (sum: f.0 * x.sum + f.1 * x.len, len: x.len)",
    );
    let op = ("op", "(a, b) => (sum: a.sum + b.sum, len: a.len + b.len)");
    let good = [
        ("id", "(1, 0)"),
        ("compose", "(f, g) => (f.0 * g.0, f.0 * g.1 + f.1)"),
        apply,
        op,
    ];
    proved(
        &Structure::action()
            .distributive()
            .check(&types, &good)
            .unwrap(),
    );
    // Composing in the other order is the bug every lazy tree has once.
    let swapped = [
        ("id", "(1, 0)"),
        ("compose", "(f, g) => (f.0 * g.0, g.0 * f.1 + g.1)"),
        apply,
        op,
    ];
    let report = Structure::action().check(&types, &swapped).unwrap();
    assert!(report.verdict("ident").is_proved());
    let counterexample = report
        .verdict("composes")
        .counterexample()
        .expect("a counterexample");
    assert_eq!(
        counterexample.to_string(),
        "f = (0, 0), g = (0, 1), a = (sum: 0, len: 1)"
    );
}

#[test]
fn assignment_is_an_action_with_an_optional_tag() {
    // Range assign: the tag is ?Int, none means "no pending update".
    let types = [("T", "(sum: Int, len: Int)"), ("F", "?Int")];
    let fields = [
        ("id", "none"),
        ("compose", "(f, g) => if f == none: g else: f"),
        (
            "apply",
            "(f, x) => if f == none: x else: (sum: f! * x.len, len: x.len)",
        ),
        ("op", "(a, b) => (sum: a.sum + b.sum, len: a.len + b.len)"),
    ];
    proved(
        &Structure::action()
            .distributive()
            .check(&types, &fields)
            .unwrap(),
    );
}

// ---- beyond linear arithmetic, and malformed instances --------------------------

#[test]
fn sampling_settles_what_the_solver_cannot() {
    // Degree four is beyond the solver; sampling settles it.
    let fields = [("op", "(a, b) => (a * a + b * b) * (a * a + b * b)")];
    let report = Structure::semigroup()
        .commutative()
        .check(&[("T", "Int")], &fields)
        .unwrap();
    assert!(report.verdict("commutes").is_proved());
    assert!(matches!(report.verdict("assoc"), Verdict::Refuted(_)));
    // An unbound type parameter is an error, not a verdict.
    let err = Structure::monoid()
        .check(&[], &[("id", "0"), ("op", "(a, b) => a + b")])
        .unwrap_err();
    assert_eq!(err.to_string(), "the type `T` is not bound");
    let err = Structure::monoid()
        .check(&[("T", "Int")], &[("id", "0")])
        .unwrap_err();
    assert_eq!(err.to_string(), "`op` is not declared");
}

// ---- the `@law`s of a file -------------------------------------------------

#[test]
fn a_valid_law_of_a_file_is_proved_and_a_broken_one_is_refuted() {
    let report = cpl_prover::laws::check_source(
        "fn add(a: Int, b: Int) -> Int = a + b\n\
         const K: Int = 3\n\
         @law comm(a: Int, b: Int) = add(a, b) == add(b, a)\n\
         @law shift(a: Int) = add(a, K) == a + 3\n\
         @law sub_comm(a: Int, b: Int) = a - b == b - a\n",
    );
    assert!(report.verdict("comm").is_proved(), "{report}");
    assert!(report.verdict("shift").is_proved(), "{report}");
    assert!(!report.holds());
    let broken = report.law("sub_comm").unwrap();
    assert!(matches!(broken.verdict, Verdict::Refuted(_)), "{report}");
    assert!(broken.broken.is_some());
}

#[test]
fn a_law_about_a_fn_the_prover_cannot_read_is_not_proved() {
    let report = cpl_prover::laws::check_source(
        "fn clamp(x: Int) -> Int:\n    if x < 0: return 0\n    return x\n\
         @law idem(x: Int) = clamp(clamp(x)) == clamp(x)\n",
    );
    assert!(matches!(report.verdict("idem"), Verdict::Unknown(_)), "{report}");
}
