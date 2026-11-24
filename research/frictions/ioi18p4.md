# Friction report: ioi18p4

## Dream

```cp
@grader fn create_circuit(m: Int, a: [Seq]1..=m) -> (c: [0..=m]Exit, switches: List<(x: Exit, y: Exit)>)
let tree = SwitchTree(leaves: 1 << bit_len(n))       // switches allocated in preorder
let used = Leaf.map(bit_rev).filter(p => Int(p) >= unused)
for (k, p) in used.items: tree.leaf[p] = if k < n: Trigger(a[Seq(k)]) else: Origin
print(tree.build(collapse: span => Int(span.r) <= unused, into: root))
```

## The grader lives in the solution

The task is a grader function (`create_circuit`). The port reads `M n A` and prints `C`, `X`
and `Y` itself, so `Exit` has to be turned into the judge's signed integers by hand (`code`).
A `@grader` entry point that takes typed arguments and serialises a returned enum would
remove `code` and the three prints.

## No std bit reversal

`bitrev` is a hand-written loop over `0..depth`. A std `bit_rev(x, bits)` (or a `Leaf`
method for power-of-two domains) is the one spelling every FFT and butterfly port wants.

## Preorder allocation needs a placeholder

`build` pushes `(x: root, y: root)` to reserve the switch's preorder number, recurses, then
overwrites it. A std builder that hands out preorder ids for a recursive split of a
`Range<Leaf>` (as `SegTree` does internally) would drop the placeholder and the index `s`.

## Verification

30 + 10 random cases agree with the v0.2 port, and the max case (M = 1e5, n = 2e5) is
byte-identical and runs in 0.04 s. The accepted C++ (`corpus/submissions/4724581.cpp`)
is a grader function without `main`, so it cannot be run on stdin here.
