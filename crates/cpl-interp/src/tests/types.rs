//! Programs whose run depends on how the checker typed them: traits, domains, rows and generics.

use super::*;

#[test]
fn seq_and_indexed_traits_unify_sequences_and_reject_non_sequences() {
    let ok_src = "\
use std.seq.{runs, prod, is_sorted}
use std.strings.{prefix_function, z_function}

fn seq_len<S: Seq>(s: S) -> Int: s.len
fn second_elem<S: Indexed>(s: S) -> Int: s.at(1)

let s: Str = \"aabba\"
let xs = [1, 1, 2, 1, 1]
domain V = 1..=5
let arr: [V]Int = [1, 1, 2, 1, 1]
let r = 1..=5

print(seq_len(s), seq_len(xs), seq_len(arr), seq_len(r))
print(second_elem(xs), second_elem(arr), second_elem(r))
print([x.len for x in s.runs()])
print(prefix_function(s))
print(prefix_function(arr))
print(prefix_function(r))
print(z_function(arr))
print(s.first!, s.last!, r[2], sum(r), prod(r), yes(is_sorted(r)))
";
    assert_eq!(
        outcome(ok_src, ""),
        (
            "5 5 5 5\n1 1 2\n2 2 1\n0 1 0 0 1\n0 1 0 1 2\n0 0 0 0 0\n5 1 0 2 1\na a 2 15 120 YES\n"
                .to_string(),
            None
        )
    );

    for (src, want) in [
        (
            "fn seq_len<S: Seq>(s: S) -> Int: s.len\nprint(seq_len(42))\n",
            "`Int` does not implement `Seq`",
        ),
        (
            "fn at0<S: Indexed>(s: S) -> Int: s.at(0)\nprint(at0(42))\n",
            "`Int` does not implement `Indexed`",
        ),
    ] {
        let mut db = cpl_check::Session::new();
        db.set_file("t.cp", src);
        let analysis = db.analyze("t.cp").expect("the file was just set");
        let diags = cpl_ir::lower(&analysis).expect_err("bound violation must fail lowering");
        assert!(
            diags.iter().any(|d| d.message.contains(want)),
            "expected {want}, got: {diags:?}"
        );
    }
}

#[test]
fn int_literal_coerces_to_domain_position() {
    let src = "\
domain V = 1..=4
let a = [V] v => Int(v) * 2
var pre: [V.cuts]Int = 0
let v = V(2)
print(a[3], pre[0], v + 1)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "6 0 3\n");
}

#[test]
fn builtin_seq_trait_param_monomorphizes() {
    let src = "\
fn total<T>(xs: Seq<T>) -> Int:
  var s = 0
  for x in xs:
    s += x
  s

let a = [2, 3, 5]
print(total(a))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "10\n");
}

#[test]
fn record_spread_prefix_and_row_bound_unify() {
    let src = "\
type Ext<V, R> = (to: V, ..R without (u, v))
fn make<V, R: (u: V, v: V, ..)>(r: R) -> Ext<V, R>:
  (to: r.v, ..r without (u, v))

domain V = 0..3
let a = make((u: V(0), v: V(2), w: 9))
print(a.to, a.w)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "2 9\n");
}

#[test]
fn opt_compare_hints_some_array_literal() {
    let src = "\
domain X = 0..2
let a: ?[X]Bool = Some([false, true])
print(if a == Some([false, true]): 1 else: 0)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "1\n");
}

#[test]
fn generic_enum_qualified_variant() {
    let src = "\
enum Boxed<T> = One(T) | Empty

fn wrap<T>(x: T) -> Boxed<T>:
  Boxed.One(x)

let b = wrap(42)
match b:
  Boxed.One(k) => print(k)
  Boxed.Empty => print(-1)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "42\n");
}

#[test]
fn bare_op_field_on_generic_struct_passes_to_monoid_of() {
    let src = "\
use std.algebra.Monoid

struct Ring<A, M>(add: A, mul: M)

fn combine<R>(a: Int, b: Int, over: R = Ring(min, +)) -> (Int, Int):
  let add = Monoid<Int>.of(over.add)
  let mul = Monoid<Int>.of(over.mul)
  (add.op(a, b), mul.op(a, b))

let (x, y) = combine(3, 5)
print(x, y)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "3 8\n");
}

const SUB: &str = "
struct Sub(bits: Int)

impl Index for Sub:
    let count = 1 << 4
    @get fn ord(self) -> Int = bits
    fn nth(k: Int) -> Sub = Sub(k)
";

#[test]
fn vm_a_walk_over_a_library_domain_matches_the_same_walk_over_ints() {
    let walk = format!(
        "{SUB}
var dp: [Sub]Int = 0
for k in 0..Sub.count:
    let s = Sub.nth(k)
    dp[s] = s.bits * s.bits % 7
for i in 0..4:
    for k in 0..Sub.count:
        let s = Sub.nth(k)
        if (s.bits >> i) & 1 == 1:
            dp[s] = (dp[s] + dp[Sub.nth(s.bits ^ (1 << i))]) % 1000000007
for s in Sub: print(dp[s])
"
    );
    let ints = "
var dp: [16]Int = 0
for k in 0..16: dp[k] = k * k % 7
for i in 0..4:
    for k in 0..16:
        if (k >> i) & 1 == 1:
            dp[k] = (dp[k] + dp[k ^ (1 << i)]) % 1000000007
for k in 0..16: print(dp[k])
";
    assert_eq!(outcome(&walk, ""), outcome(ints, ""));
}

#[test]
fn vm_a_checked_position_does_not_vouch_for_its_neighbour() {
    let source = format!(
        "{SUB}
var t = 0
for k in 0..Sub.count:
    let s = Sub.nth(k)
    let u = Sub.nth(k + 1)
    t += s.bits + u.bits
print(t)
"
    );
    let msg = failure(&source, "");
    assert!(msg.contains("has no position 16"), "unexpected trap: {msg}");
}
