// A generic only `-> yields T` names takes what the body yields: `present` over `?Range<Int>`
// pieces yields `Range<Int>`, and a user generator over `?(Int, Int)` yields the pairs.
fn somes<S: Seq, T>(xs: S) -> yields T:
    for x in xs:
        if let y = x: yield y
let a: [4]Int = [1, 2, 3, 4]
print([r.len for r in a.minimal(Window(+), ok: w => (w.all ?? 0) >= 5).present()])
let ps: List<?(Int, Int)> = [Some((1, 2)), None, Some((3, 4))]
print([p.0 + p.1 for p in somes(ps)])
