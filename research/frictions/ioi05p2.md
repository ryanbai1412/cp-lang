# ioi05p2 — Mountain

The port now follows the dream: `Rail.range.pieces(cuts:)` over rail cuts, `piece.covering(a..=b)`, a lazy `SegTree` of `Rise` under an assign-slope `Action`, and `right_while` for the ride. The remaining gaps are below.

## Enum variants with io tags
- **Where:** port lines 16-24: a `while true` loop reads a `Char` tag and pushes `Cmd.Set` or
  `Cmd.Ask` by hand, where the dream reads `read stream Cmd until End`. `@tag("I")` (spec.cp)
  and `Set(...) = 'I'` (dream) both fail to parse today.
- **What felt wrong:** the enum already names the variants; the reader repeats them.
- **Goal it hurts:** express.
- **Ideal code:** `let cmds = read stream Cmd until End`.
- **General fix:** io tags on enum variants so `read<Cmd>()` dispatches on the tag (ioi01p1 too).
- **Severity:** medium.

## The emitted C++ is over the 32 KB judge target
- **Where:** `cpl build` warns W0201: 37304 bytes for the generic judge; `cpl lint` is silent.
- **What felt wrong:** a 50-line port with one lazy tree cannot be submitted where a 32 KB source
  cap applies, and nothing in the port says which std piece costs the bytes.
- **Goal it hurts:** predictable path onto the judge.
- **General fix:** emit only the reached methods of `SegTree`/`Action`, and name the largest
  contributors in W0201.
- **Severity:** medium.

## `flat_map` loses the element type of the closure's list
- **Where:** `cmds.flat_map(c => match c: Set(a, b, _) => [a.l, b.r] _ => [])`.
- **What felt wrong:** E0302 "`flat_map` does not work on the types it was given here:
  expected `Int`, found `Rail.cuts`" (`xs.flat_map(x => [V(x).l, V(x).r])` alone repeats it);
  the comprehension `[x for c in cmds if let ... for x in [a.l, b.r]]` keeps the type.
- **Goal it hurts:** express.
- **General fix:** bind `flat_map`'s yielded `V` from `U: Seq`'s element. Already routed:
  compiler worklog, "Request from stdlib" `flat_map` entry.
- **Severity:** low (the comprehension is as clear).

## A flipped `compose` type-checks and answers wrong
- **Where:** `compose: (old, new) => new ?? old` (spec.cp had the names in the wrong order;
  `compose(new, old)` is the contract).
- **What felt wrong:** the tree kept the oldest slope; the 8 old tests still passed, and 30 of 60
  generated inputs disagreed with the AC submission (one trapped on `/ 0`).
- **Goal it hurts:** hard-to-get-wrong.
- **General fix:** stress checks `laws_hold` on actions handed to `SegTree(act:)`; routed to
  tools (worklog 2026-11-23, with a 4-leaf repro).
- **Severity:** high.

## Tests 01-08 break the statement
The factory tests set slopes like `I 1 2 -8`, so elevations go negative, outside the statement's
"[0, 1e9] at any moment". They are kept; `gen.py` and tests 09-17 stay inside it (09 is the
statement sample).

## Measurements
20000-line generated input (n = 1e9): AC C++ 0.058 s, `cpl run` 1.17 s, same output.
