//! Associated types chosen by impl (R08.1.9, decision 456): `M.Fold` in a
//! generic fn's signature is the type `M`'s impl chooses.

use super::*;

const OP: &str = "\
trait Op<T>:
    type Fold
    fn fold2(self, a: T, b: T) -> Self.Fold
struct Mono<T>(id: T, f: (T, T) -> T)
struct Bare<T>(f: (T, T) -> T)
impl<T> Op<T> for Mono<T>:
    type Fold = T
    fn fold2(self, a: T, b: T) -> T = self.f(a, b)
impl<T> Op<T> for Bare<T>:
    type Fold = ?T
    fn fold2(self, a: T, b: T) -> ?T = Some(self.f(a, b))
fn go<M: Op<Int>>(m: M) -> M.Fold = m.fold2(2, 3)
";

#[test]
fn a_generic_fn_returns_the_type_its_impl_chooses() {
    let ok = format!(
        "{OP}let z: Int = go(Mono(0, (a, b) => a + b))\nlet w: ?Int = go(Bare((a: Int, b: Int) => a * b))\n"
    );
    assert_eq!(check(&ok), vec![]);
    let bad = format!(
        "{OP}let z: Str = go(Mono(0, (a, b) => a + b))\nlet w: Int = go(Bare((a: Int, b: Int) => a * b))\n"
    );
    let found = check(&bad);
    assert_eq!(found.len(), 2, "{found:#?}");
    assert!(found.iter().all(|d| d.code.0 == "E0302"), "{found:#?}");
    assert!(found[0].message.contains("Int"), "{}", found[0].message);
    assert!(found[1].message.contains("?Int"), "{}", found[1].message);
}

const BOX: &str = "\
trait Box:
    type Item
    fn get(self) -> Self.Item
struct IntBox: v: Int
";

#[test]
fn an_impl_chooses_every_associated_type_and_no_other() {
    let missing = format!("{BOX}impl Box for IntBox:\n    fn get(self) -> Int = self.v\n");
    let found = check(&missing);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].code.0, "E0606");
    assert_eq!(
        found[0].message,
        "`impl Box for IntBox` does not choose `Item`"
    );
    let extra = format!(
        "{BOX}impl Box for IntBox:\n    type Item = Int\n    type Elem = Int\n    fn get(self) -> Int = self.v\n"
    );
    let found = check(&extra);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].code.0, "E0203");
    assert_eq!(found[0].message, "`Box` declares no associated type `Elem`");
}

#[test]
fn a_generic_body_sees_only_the_projection() {
    let src = format!(
        "{BOX}impl Box for IntBox:\n    type Item = Int\n    fn get(self) -> Int = self.v\nfn as_int<B: Box>(b: B) -> Int = b.get()\nfn same<B: Box>(b: B) -> B.Item = b.get()\n"
    );
    let found = check(&src);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].code.0, "E0302");
    assert!(found[0].message.contains("B.Item"), "{}", found[0].message);
}

#[test]
fn self_names_the_impls_own_choice() {
    let src = format!(
        "{BOX}impl Box for IntBox:\n    type Item = Int\n    fn get(self) -> Self.Item = self.v\nlet s: Str = IntBox(1).get()\n"
    );
    let found = check(&src);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].code.0, "E0302");
}

const PICK: &str = "\
trait Pick:
    type Found
    fn pick(self, x: Int) -> Self.Found
impl Pick for Int -> Bool:
    type Found = Bool
    fn pick(self, x: Int) -> Bool = self(x)
fn go<P: Pick>(p: P) -> P.Found = p.pick(3)
";

#[test]
fn a_fn_type_impl_chooses_its_own_associated_type() {
    assert_eq!(
        check(&format!("{PICK}let b: Bool = go((v: Int) => v > 2)\n")),
        vec![]
    );
    let found = check(&format!("{PICK}let b: Int = go((v: Int) => v > 2)\n"));
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].code.0, "E0302");
}

#[test]
fn a_generic_fn_type_impl_binds_its_params_from_the_fn() {
    let op = OP
        .replace(
            "impl<T> Op<T> for Bare<T>:",
            "impl<T> Op<T> for (T, T) -> T:",
        )
        .replace("Some(self.f(a, b))", "Some(self(a, b))");
    let ok = format!(
        "{op}let z: Int = go(Mono(0, (a, b) => a + b))\nlet w: ?Int = go((a: Int, b: Int) => a * b)\n"
    );
    assert_eq!(check(&ok), vec![]);
    let found = check(&format!("{op}let w: Int = go((a: Int, b: Int) => a * b)\n"));
    assert_eq!(found.len(), 1, "{found:#?}");
    assert!(found[0].message.contains("?Int"), "{}", found[0].message);
}

#[test]
fn an_impls_choice_has_the_traits_its_bound_names() {
    let src = "\
trait Keyed:
    type K: Ord
    fn key(self) -> Self.K
struct P(a: Int)
impl Keyed for P:
    type K = Int -> Int
    fn key(self) -> Int -> Int = x => x + self.a
";
    let found = check(src);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].code.0, "E0606");
    assert!(found[0].message.contains("`Ord`"), "{}", found[0].message);
    assert_eq!(
        check(
            &src.replace("Int -> Int", "Int")
                .replace("x => x + self.a", "self.a")
        ),
        vec![]
    );
}
