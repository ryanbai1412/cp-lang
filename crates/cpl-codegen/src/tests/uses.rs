//! The Uses table: what may happen to each heap binding.

use super::lower;
use crate::uses::Home;

fn uses_of(source: &str, name: &str) -> crate::uses::Use {
    let target = crate::Target::default();
    let program = crate::prepared(&lower(source), &target);
    let slot = program
        .main_locals
        .iter()
        .position(|l| l.name == name)
        .expect("the local is in main");
    crate::emit::Emitter::new(&program, &target)
        .uses
        .of_binding(Home::Top, slot)
}

#[test]
fn a_list_read_only_by_index_is_neither_mutated_nor_moved() {
    let source = "let n = read<Int>()
for v in 0..n:
    let xs = [v, v + 1]
    print(xs[0] + xs.len)
";
    let xs = uses_of(source, "xs");
    assert!(!xs.mutated && !xs.moved, "{xs:?}");
}

#[test]
fn a_pushed_list_is_mutated_and_one_read_whole_may_move() {
    let source = "let n = read<Int>()
for v in 0..n:
    var xs = [v]
    xs.push(v)
    let ys = xs
    print(ys.len)
";
    let xs = uses_of(source, "xs");
    assert!(xs.mutated && xs.moved, "{xs:?}");
}

#[test]
fn a_read_of_a_value_set_while_it_lives_is_a_copy() {
    let out = super::cpp(
        "var best = (gap: 9, id: 0)
let a = best min= (gap: 4, id: 7)
let b = best min= (gap: 4, id: 8)
print(best.gap, best.id, Int(a), Int(b))
",
    );
    assert!(
        !out.contains("std::tuple<ll, ll>& t"),
        "`min=` compares with what `best` was, so that is copied, not lent:\n{out}"
    );
}

#[test]
fn a_variant_payload_read_only_is_lent_while_the_others_are_read() {
    let out = super::cpp(concat!(
        "enum Op = Put(x: Int) | Ask(s: Str, k: Int)\n",
        "let q: 1..=10 = read()\n",
        "for op in read<[q]Op>():\n",
        "    match op:\n",
        "        Put(x) => print(x)\n",
        "        Ask(s, k) => print(s.len + k)\n",
    ));
    assert!(
        !out.contains("std::string s = op."),
        "reading `k` off `op` reads a part of it, so `s` is lent:\n{out}"
    );
}

#[test]
fn a_name_another_fn_lent_does_not_stop_this_fn_moving_its_own() {
    let out = super::cpp(concat!(
        "fn first(rows: List<List<Int>>) -> Int:\n",
        "    let r = rows[0]\n",
        "    r.len\n",
        "fn build(n: Int) -> List<Int>:\n",
        "    var xs = List.filled(n, 0)\n",
        "    xs.push(1)\n",
        "    xs\n",
        "let n: 1..=10 = read()\n",
        "let ys = build(n)\n",
        "let t = [i for i in 0..n]\n",
        "let zs = t\n",
        "print(first([ys, zs]), zs.len)\n",
    ));
    assert!(
        out.contains("= std::move("),
        "a temporary this fn built moves into its name:\n{out}"
    );
}

fn copy_reasons(source: &str) -> Vec<&'static str> {
    crate::explain_copies(&lower(source))
        .into_iter()
        .map(|(_, _, why)| why)
        .collect()
}

#[test]
fn a_copy_names_the_reason_the_table_found() {
    assert_eq!(
        copy_reasons(
            "fn f(rows: List<List<Int>>) -> Int:
    var own = rows[0]
    own.push(5)
    own.len
print(f([[1], [2]]))
"
        ),
        [crate::uses::SET_AGAIN]
    );
}

#[test]
fn a_loop_that_walks_a_copy_of_a_place_lists_the_copy() {
    assert_eq!(
        copy_reasons(
            "let n: Int = read()
let sieve = Sieve(n)
for a in sieve.primes:
    if sieve.is_prime(2 * n - a):
        print(a)
"
        ),
        [crate::uses::WALKED_COPY]
    );
}

#[test]
fn a_value_that_owns_no_heap_memory_is_never_listed_as_a_copy() {
    let modular = "type Z = Mod<1000000007>\nlet x: Z = 3\nprint(x ** 4, x ** 0)\n";
    assert_eq!(copy_reasons(modular), [] as [&str; 0]);
    let pair = "fn f(ps: List<(Int, Int)>) -> Int:\n    var p = ps[0]\n    p.0 += 1\n    p.0\nprint(f([(1, 2)]))\n";
    assert_eq!(copy_reasons(pair), [] as [&str; 0]);
    let holder = "struct H:\n    xs: List<Int>\nfn f(hs: List<H>) -> Int:\n    var h = hs[0]\n    h.xs.push(1)\n    h.xs.len\nprint(f([H(xs: [1])]))\n";
    assert_eq!(copy_reasons(holder), [crate::uses::SET_AGAIN]);
}

#[test]
fn a_name_destructured_from_a_fresh_tuple_is_moved_not_copied() {
    let source = "fn f() -> Int:\n    var (a, b) = ([1], [2])\n    a.push(3)\n    a.len + b.len\nprint(f())\n";
    assert_eq!(copy_reasons(source), [] as [&str; 0]);
}

#[test]
fn a_while_let_tuple_payload_is_moved_not_copied() {
    let source = "var xs = [(2, \"bb\")]\nwhile let (k, s) = xs.pop():\n    print(k, s)\n";
    assert_eq!(copy_reasons(source), [] as [&str; 0]);
}

#[test]
fn part_of_a_call_result_is_moved_not_copied() {
    let source = "let (n, s) = \"4 x\".parse<(Int, Str)>()!\nprint(n, s)\n";
    assert_eq!(copy_reasons(source), [] as [&str; 0]);
}

#[test]
fn a_clone_or_sorted_of_a_place_is_listed_as_a_copy() {
    let source = "var a = [3, 1, 2]\nvar b = a.clone()\nb.push(4)\nprint(a, b)\n";
    assert_eq!(copy_reasons(source), [crate::uses::CLONED]);
    let twin = "var a = [3, 1, 2]\nlet c = a.sorted()\nlet d = [2, 1].sorted()\nprint(a, c, d)\n";
    assert_eq!(copy_reasons(twin), [crate::uses::SORTED]);
}

#[test]
fn explain_reads_closures_copies_and_dropped_fields_off_one_write() {
    let program = lower(
        "struct S:\n    f: (Int) -> Int\n    unused: Int\nvar a = [3, 1, 2]\nvar b = a.clone()\nb.push(4)\nlet s = S(f: x => x + 1, unused: 0)\nprint(s.f(b[0]))\n",
    );
    let once = crate::explain_lowered(&program);
    assert_eq!(
        format!("{:?}", once.closures),
        format!("{:?}", crate::explain_closures(&program))
    );
    assert_eq!(once.copies, crate::explain_copies(&program));
    assert_eq!(
        once.dropped,
        crate::devirt::explain_dropped_fields(&program)
    );
}

#[test]
fn a_fn_run_once_moves_a_global_nothing_reads_after_it() {
    let out = super::cpp(concat!(
        "let n: 1..=10 = read()\n",
        "let base = [i for i in 0..n]\n",
        "fn build() -> List<Int>:\n",
        "    var xs = base\n",
        "    xs.push(1)\n",
        "    xs\n",
        "let built = build()\n",
        "print(built)\n",
    ));
    assert!(
        out.contains("= std::move(base)"),
        "`base` is not read after `build` takes it, so it moves:\n{out}"
    );
}

#[test]
fn a_global_read_after_the_fn_that_takes_it_is_copied() {
    let out = super::cpp(concat!(
        "let n: 1..=10 = read()\n",
        "let base = [i for i in 0..n]\n",
        "fn build() -> List<Int>:\n",
        "    var xs = base\n",
        "    xs.push(1)\n",
        "    xs\n",
        "let built = build()\n",
        "print(built, base)\n",
    ));
    assert!(
        !out.contains("std::move(base)"),
        "`base` is printed after `build`, so `build` copies it:\n{out}"
    );
}

#[test]
fn a_read_only_call_on_a_global_a_closure_writes_copies_nothing() {
    let out = super::cpp(concat!(
        "let n: 1..=10 = read()\n",
        "domain X = 0..n\n",
        "var t = SegTree<X, Int>(max, fill: 0)\n",
        "fn best(at: X) -> Int = t.fold(X.first!..=at)\n",
        "fn add(at: X):\n",
        "    t.set(at, best(at) + 1)\n",
        "let f = (x: X) => add(x)\n",
        "for x in X: f(x)\n",
        "print(t.fold(X.first!..=X.last!))\n",
    ));
    assert!(
        !out.contains("(t),"),
        "`fold` calls only the `Int` op, never `f`, so it can't change `t` and reads it in place:\n{out}"
    );
}

#[test]
fn a_call_that_may_run_a_closure_writing_the_global_copies_it() {
    let out = super::cpp(concat!(
        "let n: 1..=10 = read()\n",
        "domain X = 0..n\n",
        "var t = SegTree<X, Int>(max, fill: 0)\n",
        "fn bump(a: Int, b: Int) -> Int:\n",
        "    t.set(X.first!, a)\n",
        "    max(a, b)\n",
        "let g = (a: Int, b: Int) => bump(a, b)\n",
        "fn best(at: X) -> Int = t.fold(X.first!..=at)\n",
        "print(g(1, 2), best(X.last!))\n",
    ));
    assert!(
        out.contains("(t),"),
        "`g` has the op's type and writes `t`, so `fold` may change `t` and reads a copy:\n{out}"
    );
}

#[test]
fn binding_a_lent_optional_copies_nothing() {
    let source = "use std.persistent.PersistentSegTree\ndomain N = 0..4\nlet z: [N]Int = [0, 0, 0, 0]\nvar t = PersistentSegTree<N, Int>(+, a: z)\nlet v1 = t.set(t.base, N(1), 1)\nprint(yes(t.diff_kth(v1, t.base, 0) == Some(N(1))))\n";
    assert!(crate::tests::cpp(source).contains("cpl::Lent<"));
    assert_eq!(copy_reasons(source), [] as [&str; 0]);
}

#[test]
fn a_library_fold_that_gives_its_identity_on_an_empty_range_lists_no_copy() {
    let src = concat!(
        "use std.bag.{AugSortedBag}\n",
        "var s: AugSortedBag<Int, Str> = AugSortedBag((x, y) => x + y, \"\", k => Str(k))\n",
        "s.add(3)\n",
        "let lo: Int = read()\n",
        "print(s.fold_between(lo, 9))\n",
    );
    assert!(super::cpp(src).contains("return cpl::held_ref(self.id);"));
    let whats: Vec<String> = crate::explain_copies(&lower(src))
        .into_iter()
        .map(|(_, what, _)| what)
        .collect();
    assert_eq!(whats, [] as [&str; 0]);
}

#[test]
fn a_last_read_assigned_inside_an_if_moves() {
    let out = super::cpp(concat!(
        "let n: 1..=10 = read()\n",
        "var best: ?List<Int> = None\n",
        "for i in 0..n:\n",
        "    let w = [j * i for j in 0..n]\n",
        "    if best == None or w < best!: best = w\n",
        "var dp = [0 for _ in 0..n]\n",
        "for i in 0..n:\n",
        "    let next = [dp[j] + i for j in 0..n]\n",
        "    if i % 2 == 0:\n",
        "        dp = next\n",
        "print(best!, dp)\n",
    ));
    assert!(
        out.contains("(std::move(w))") && out.contains("dp = std::move(next)"),
        "`w` and `next` are last read where they are assigned, so they move:\n{out}"
    );
}

#[test]
fn a_list_sorted_by_a_key_at_its_last_use_lists_no_copy() {
    let source = "let n: 2..=10 = read()\nlet xs: [n](t: Int, x: Int) = read()\nprint(xs.sorted(by: o => o.t).first!.x)\n";
    assert_eq!(copy_reasons(source), [] as [&str; 0]);
}

#[test]
fn a_clone_written_in_a_library_fn_is_listed_at_its_fn() {
    let src = "use std.seq.{inversions}\nlet xs = [3, 1, 2]\nprint(inversions(xs))\n";
    assert!(super::cpp(src).contains("std::vector<ll> tmp = std::vector<ll>(cur);"));
    let whats: Vec<String> = crate::explain_copies(&lower(src))
        .into_iter()
        .map(|(_, what, _)| what)
        .collect();
    assert_eq!(whats, ["a read in inversions"]);
}

#[test]
fn an_if_let_over_a_place_written_after_its_last_read_is_lent() {
    let source = concat!(
        "fn longest(xs: List<Str>) -> ?Str:\n",
        "    var best: ?Str = None\n",
        "    for x in xs:\n",
        "        if let b = best:\n",
        "            if x.len > b.len: best = Some(x)\n",
        "        else: best = Some(x)\n",
        "    best\n",
        "print(longest(read<List<Str>>())!)\n",
    );
    assert_eq!(copy_reasons(source), [] as [&str; 0]);
    let out = super::cpp(source);
    assert!(
        !out.contains("std::optional<std::string> t0 = best;"),
        "`best` is set only after `b`'s last read, so the `if let` reads it in place:\n{out}"
    );
}
