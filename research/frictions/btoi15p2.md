# btoi15p2: friction report

Port: `corpus/problems/btoi15p2/port.cp`. There is no `dream.cp`; the dream is below.

## Measurements

- Tokens: port 264, accepted C++ 919.
- Emitted C++: 14084 bytes.
- `--max` (n = 3e5, median of 15): port 29.7 ms / 27.1 MB, accepted 165.4 ms / 55.2 MB (0.18x time).
- Output: identical to the accepted C++.

## Dream

```cp
let n: 1..=3e5 = read()
domain Op = 0..n
domain Lg = 0..=log2(n)
let ops: [Op]-n..=n = read()
fn level(i: Op) -> Int = max(0, -ops[i])

// up[j, i]: 2^j steps down the chain under undo i
var up: [Lg, Op]?Op = None

// the topmost operation of level < x in the chain whose top is `top`
fn below(top: ?Op, x: Int) -> ?Op:
    var u = top?
    if level(u) < x: return u
    for j in Lg.rev:
        if let v = up[j, u] and level(v) >= x: u = v
    up[Lg.first!, u]

for i in Op:
    if ops[i] < 0:
        up[Lg.first!, i] = below(below(i.prev, level(i))!.prev, level(i))
        for j in Lg.drop(1):
            up[j, i] = up[j.prev!, up[j.prev!, i]?]   // hypothetical: `?` ends the fill
    print(below(i, 1).map(t => ops[t]) ?? 0)
```

## Friction 1: the jump depth `Lg = 0..19` is a magic number

- Where: `domain Lg = 0..19`.
- What felt wrong: 19 is "enough bits for 3e5"; the fact comes from `n` and is restated
  as a literal.
- Goal it hurts: R4.
- Ideal code: `domain Lg = 0..=log2(n)` (or `0..=n.bit_len`), derived from the bound.
- General fix: port edit if `log2` / `bit_len` is usable in a domain bound; otherwise a
  Core gap. Not yet probed, so idioms checks it on the next port pass.
- Severity: low.
- Owner: idioms (probe and port edit).

## Friction 2: the doubling fill uses `continue` / `break` for two optionals

- Where: `let lower = j.prev else: continue` and `let half = up[lower, i] else: break`.
- What felt wrong: skipping `Lg.first!` and stopping at the first missing jump take two
  `let ... else` exits where the editorial says "for j >= 1 while the jump exists".
  `for j in Lg.drop(1)` would remove the first.
- Goal it hurts: R2.
- Ideal code: `for j in Lg.drop(1):` plus one `let half = up[j.prev!, i] else: break`.
- General fix: port edit.
- Severity: low.
- Owner: idioms.
