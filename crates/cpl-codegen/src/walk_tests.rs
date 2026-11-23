//! A library-domain `Iter::Walk` is expanded in lowering, before any backend.

use crate::tests::lower;

#[test]
fn no_library_domain_walk_survives_lowering() {
    let program = lower(concat!(
        "struct Sub<B>(bits: Int)\n",
        "impl Index for Sub<B>:\n",
        "    let count = 1 << B.count\n",
        "    @get fn ord(self) -> Int = bits\n",
        "    fn nth(k: Int) -> Sub<B> = Sub(k)\n",
        "domain V = 0..3\n",
        "fn total<D: Domain>() -> Int = Sub<D>.count(s => s.bits > 1)\n",
        "for s in Sub<V>.rev: print(s.bits)\n",
        "print(Sub<V>.find(s => s.bits > 2).map(s => s.bits) ?? -1, total<V>())\n",
        "print([s.bits for s in Sub<V>], Sub<V>.min(by: s => -s.bits)!.bits)\n",
        "let any6 = () => Sub<V>.any(s => s.bits == 6)\n",
        "print([s.bits for s in Sub<V>.rev if s.bits > 3], any6())\n",
    ));
    // An `Iter` only lives in a `Stmt::For` head, but scan the whole program
    // (fused loops, comprehensions, closures, tables) so a Walk anywhere fails.
    let dump = format!("{program:?}");
    assert!(
        !dump.contains("Walk {"),
        "an Iter::Walk reached the backends"
    );
    assert!(
        dump.contains("Range {"),
        "the walks should expand to ordinal ranges"
    );
}
