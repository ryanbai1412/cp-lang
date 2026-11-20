# Ugly-code friction audit 3: geometry, math, strings, simulation, greedy, constructive, interactive

Language version: CP Lang 0.2.2. Scope: `proto/ports/` at `main` (1,001 `.cp` files). This audit read
208 of them in full: every port that touches floats, points, format specs, `ask`, `@interactor`,
`@grader` or `@checker` (150 files), plus all of `scout_batch_22` (strings), `scout_batch_25`
(greedy and bitwise), `scout_batch_29` (number theory and linear algebra) and `scout_batch_31`
(grids and simulation), the `blind_critic_*` drafts, and the top-level ports in those topics. The
counts quoted below come from `grep` over all 1,001 files, so they are lower bounds, not samples.

The standard applied: anything that makes a contestant stop and think about the language instead of
the problem is a defect. "It works" is not a pass. A sketch that reads well but only because it uses
a helper that does not exist is a defect too, and so is a `today/` port that is three times longer
than its sketch.

Every quoted block starts with the file it was copied from and is verbatim (trailing comments
trimmed). Every "ideal" block is the proposed 0.2.2 spelling. Each pattern ends with a verdict:
**userland** (a `stdlib/` function, method or derived trait, no compiler change), **core** (a rule
in the checker or the grammar), or **toolchain** (the design already says the right thing and
`proto/cpc`, `proto/cpi` or the Rust front end does not do it yet). Ryan's userland-first rule is
applied strictly: a core change is proposed only where no library function can express the fix, and
each one removes a rule or reuses an existing concept instead of adding one.

The patterns are ordered by how much contest time they cost, most expensive first.

## 1. Printing a decimal number takes between 4 and 25 lines, in eight different spellings

This is the worst finding of the audit. A C++ contestant types `printf("%.3f\n", x)` without
thinking. In the ports, the same task is spelled eight ways, and the versions that actually run
today build the digits by hand.

The eight spellings, all for "print with k decimals":

```cp
// from proto/ports/scout_batch_23/noi07p1.cp
    print("{total:.3f}")
```
```cp
// from proto/ports/scout_batch_12/cco19p1.cp
print("{wins(board, J):.3}")
```
```cp
// from proto/ports/scout_batch_1/ccc05s5.cp
print(ranks / F64(t), digits: 2)
```
```cp
// from proto/ports/scout_batch_14/ccc20s5.cp
    print(suffix_sum / Float(n), prec: 8)
```
```cp
// from proto/ports/scout_batch_21/apio23p1.cp
    print(fixed(best[C(h)]!, 6))
```
```cp
// from proto/ports/scout_batch_27/noi07p1.cp
fn fixed3(x: Float) -> Str:
    let units = Int(x * 1000 + 0.5)
    let frac = units % 1000
    "{units / 1000}.{frac / 100}{frac / 10 % 10}{frac % 10}"
```
```cp
// from proto/ports/scout_batch_50/today/noi04p5.cp
write(best / 10)
write(".")
print(best % 10)
```
```cp
// from proto/ports/scout_batch_35/today/coi07p1.cp
let th = Int(t * 1000.0 + 0.5)
let frac = th % 1000
let pad = if frac < 10: "00" elif frac < 100: "0" else: ""
print("{th / 1000}.{pad}{frac}")
```

The cheat sheet says the one spelling is `"{x:.6f}"` and that a bare `print(x)` of an `F64` is an
error. `plan/16-io.md` still says `print(x, digits: 9)`. Two `today/` ports print a bare float and
would lose precision on a judge (`print(ans)` in `scout_batch_23/today/dpi.cp`, `print(Float(n))`
in `scout_batch_36/today/cco19p4.cp`); the sketch of the first one notes that the reference
interpreter drops the format spec silently. A dropped format spec is a wrong answer that passes
every local sample.

The hand-rolled versions get worse when the value is an exact fraction. This port has a rational
slope and needs five decimals:

```cp
// from proto/ports/scout_batch_29/sdoi17r1p6.cp
fn slope(num: Int, den: Int) -> Str:
    let cut = abs(num) * 1_000_000 / den
    let units = (cut + 5) / 10
    var digits = "{units % 100_000}"
    while digits.len < 5: digits = "0" + digits
    (if num < 0 and units > 0: "-" else: "") + "{units / 100_000}." + digits
```

This one binary-searches a scaled integer and prints six decimals:

```cp
// from proto/ports/scout_batch_15/sdoi17r1p4.cp
let scaled6 = (lo + 5) / 10
let whole = scaled6 / 1000000
var frac = scaled6 % 1000000
var digits = List.filled(6, 0)
for k in (0..6).rev:
    digits[k] = frac % 10
    frac /= 10
var out = Str(whole) + "."
for d in digits: out = out + Str(d)
print(out)
```

And `scout_batch_51/today/ccc20s1.cp` spends 50 lines on a 5-line problem, 11 of them on this:

```cp
// from proto/ports/scout_batch_51/today/ccc20s1.cp
fn digit_char(d: Int) -> Char:
    if d == 0: return '0'
    if d == 1: return '1'
    if d == 2: return '2'
```

Half-integers are special-cased as well:

```cp
// from proto/ports/scout_batch_32/ceoi22p5.cp
        print(if t % 2 == 1: "{t / 2}.5" else: "{t / 2}")
```

Ideal: one format mini-language, the one the cheat sheet already names, working in every tool, plus
one library function for exact fractions.

```cp
print("{total:.3f}")                      // F64, round-half-even never prints -0.000
print("{fixed(num, den, 5)}")             // exact Int fraction, no float involved
print("{fixed(t, 2, 1)}")                 // the half-integer case
```

Verdict: **toolchain** for `{x:.kf}` (make `cpi`, `cpc` and the Rust interpreter honour it, and make
all three reject `print(x)` of a bare `F64` as the cheat sheet says; delete `digits:`, `prec:` and
`write` from `plan/16-io.md` and migrate the 25 call sites). **Userland** for
`fixed(num: Int, den: Int, k: Int) -> Str` in `std.math`, which is `slope` above with a name.

## 2. Zero padding, digit characters and string repetition are written out by hand

The same family as pattern 1, for integers and strings. Zero-padding a number:

```cp
// from proto/ports/scout_batch_29/wide.cp
        var digits = "{lo}"
        while digits.len < 18: digits = "0" + digits
        "{hi}" + digits
```

Turning a digit into a character:

```cp
// from proto/ports/scout_batch_22/ccc99s5.cp
    for row in rows: print(Str([Char(Int('0') + digit[Letter(c)]!) for c in row]))
```

Repeating a string (the cheat sheet lists `"bc" * 3`, the port could not use it):

```cp
// from proto/ports/scout_batch_19/egoi23p3.cp
fn repeat_str(s: Str, k: Int) -> Str:
    var out = ""
    for _ in 0..k: out += s
    out
```

Its `today/` twin gives up on `Str` and builds `List<Char>` with a helper, so a move string such as
`"v>^"` becomes `['v', '>', '^']`:

```cp
// from proto/ports/scout_batch_19/today/egoi23p3.cp
            append_rep(mut out, ['v', '>', '^'], w - 1)
            append_rep(mut out, ['>', 'v', '>'], 1)
```

Printing a number in binary at a fixed width, and parsing hex:

```cp
// from proto/ports/scout_batch_25/noip19p1.cp
print(join(g.bit(i) for i in (0..n).rev))
```
```cp
// from proto/ports/scout_batch_22/noi21p4.cp
fn hex(c: Char) -> Int = if c.is_digit: c - '0' else: 10 + (c - 'A')
```

Ideal:

```cp
"{hi}{lo:018}"                            // width with a leading 0 pads with zeros
print("{g:0{n}b}")                        // binary, n wide; {x:x} and {x:X} for hex
out += "v>^" * (w - 1) + ">v>"
let d = c.digit(16)!                      // Char -> ?Int in a base; Char.of_digit(d) back
```

Verdict: the width, `0`, `b`, `x` flags are **toolchain** (the format spec is one concept that
already exists; it should cover the `printf` subset contestants know and nothing more). `Str * Int`
is **toolchain** (decided, not built). `Char.digit(base)` and `Char.of_digit(d)` are **userland**.

## 3. A `Bool` cannot be read from or printed as what judges actually use

Judges send `0`/`1`, `true`/`false`, `Y`/`N`, and expect `YES`/`NO`, `DA`/`NE`, `1`/`0`. The ports
convert by hand in both directions: `if c: 1 else: 0` appears 80 times in 62 files, and a
`"YES"`/`"NO"`-style literal pair appears 62 times in 39 files.

Reading:

```cp
// from proto/ports/scout_batch_25/dmopc21c7p3.cp
let lit: [V]Bool = [V] _ => read<Int>() == 1
```
```cp
// from proto/ports/scout_batch_25/today/ioi09p3.cp
let solved: [C, Task]0..=1 = read()
```
```cp
// from proto/ports/scout_batch_19/ioi07p1.cp
    let reply: Str = ask("examine", x, y, limit: 300)
    reply == "true"
```
```cp
// from proto/ports/yac7p5.cp
    let broke: 0..=1 = ask("?", egg, floor)
    broke == 1
```

Writing:

```cp
// from proto/ports/scout_batch_23/ioi17p3.cp
print([if alive[v]: 1 else: 0 for v in V])
```
```cp
// from proto/ports/scout_batch_24/dmpg15g6.cp
    print(if res[qi]: "YES" else: "NO")
```
```cp
// from proto/ports/scout_batch_47/cco07p5.cp
            sol.send(if p in l..=r: "True" else: "False")
```

And arithmetic on a truth value, where `Int(b)` exists on the cheat sheet and almost nobody found it:

```cp
// from proto/ports/scout_batch_33/ioi12p6.cp
    for k in kids[v]: wins[k] = wins[v] + (if r > inner[v]: 1 else: 0)
```
```cp
// from proto/ports/scout_batch_43/etsr23r4p1.cp
            cur[v] += ((k ^ flip) % 2) * 2 - 1
```

Ideal: `Bool` reads `0`/`1` by default, and the existing `none:` idea (one named argument that says
how a special value is spelled, symmetric on `read`, `print`, `ask` and `send`) extends to the two
truth values.

```cp
let lit: [V]Bool = read()                         // 0 / 1
let solved: [C, Task]Bool = read()
fn examine(x: Int, y: Int) -> Bool = ask("examine", x, y, yes: "true", no: "false")
print(res[qi], yes: "YES", no: "NO")
print(alive)                                      // a Bool list prints 1 0 1 ...
wins[k] = wins[v] + Int(r > inner[v])
```

Verdict: **userland**. `Read`/`Show` for `Bool` and the `yes:`/`no:` named arguments live in the
prelude next to `none:`. Add a lint that rewrites `if c: 1 else: 0` to `Int(c)`.

## 4. `ask` and `read` only work on the right of a typed `let`

Every interactive port wraps `ask` in a three-line function whose only job is to give the reply a
type. A typed `let` whose only job is to type an `ask` appears 16 times in 13 files:

```cp
// from proto/ports/yac6p3.cp
fn nearest(x: Int, y: Int) -> Int:
    let d: 0..=2e9 = ask("?", x, y)
    d
```
```cp
// from proto/ports/scout_batch_44/dmopc21c2p3.cp
fn probe(guess: Int) -> Int:
    let d: Int = ask(guess, limit: 31_999)
    d
```
```cp
// from proto/ports/scout_batch_34/dmopc20c6p6.cp
fn pour_jug(x: Jug, y: Jug) -> Int:
    let moved: Int = ask("D", x, y, limit: 4 * n)
    moved
```

`read` has the same hole. Twelve files define this, and the "read n values into a list" loop that
follows from it is everywhere in `today/`:

```cp
// from proto/ports/scout_batch_52/today/noi02p2.cp
fn ri() -> Int:
    let x: Int = read()
    return x
```
```cp
// from proto/ports/scout_batch_23/today/dpi.cp
var p: List<Float> = []
for _ in 0..n:
    let x: Float = read()
    p.push(x)
```
```cp
// from proto/ports/scout_batch_21/noi07p2.cp
for d in Day:
    let (pa, pb, r): (Float, Float, Float) = read()
    (a[d], b[d], rate[d]) = (pa, pb, r)
```

Three spellings of "read with an explicit type" are in use: `read<Int>()`
(`scout_batch_25/dmopc21c7p3.cp`), `read::<Int>()` (`scout_batch_49/ioi01p3.cp`) and the wrapper.

Ideal: `read()` and `ask(...)` take their type from wherever the value is going, like any other
expression whose type is inferred from context: a `let` annotation, a return type, a typed place on
the left of `=`, a function argument. Where there is no context, the one explicit spelling is
`read::<T>()`, which `decisions.md` already fixed for generic calls.

```cp
fn nearest(x: Int, y: Int) -> Int = ask("?", x, y)
let p: [0..n]F64 = read()
(a[d], b[d], rate[d]) = read()
var skip = read::<Int>() - 1
```

Verdict: **core**, and a small one. It is the checker rule "the expected type flows into `read` and
`ask`" applied at every typed position instead of at one. No new syntax and no new concept; it
deletes the special case that makes `let x: T = read()` the only legal form.

## 5. Converting between an index and a number has nine spellings and 900 call sites

The cheat sheet names two conversions: `v.to_idx` and `V.from_idx(i) -> ?V`. The ports use nine:
`.to_idx` (323 uses in 141 files), `.idx` (181 in 83), `.label` (126 in 71), `.ord` (52 in 30),
`D.from_idx(i)!` (161 in 32), `D.nth(i)` (63 in 31), `D(i)` (over 200), `D.at(i)` and `D.try(i)`.
Some mean the 0-based position, some mean the written label (`1` for the first element of
`1..=n`), and a reader cannot tell which without knowing which batch wrote the file.

The cost is not only the drift. When a problem's arithmetic is on positions, the conversions drown
the algorithm:

```cp
// from proto/ports/scout_batch_24/today/ceoi22p1.cp
    while stk.len > 0 and deck[Pos.nth(stk[stk.len - 1])].to_idx < deck[p].to_idx:
```
```cp
// from proto/ports/scout_batch_24/today/ceoi22p1.cp
    ans[qi] = deck[Pos.nth(pos_of[leader].to_idx + off)].to_idx + 1
```
```cp
// from proto/ports/blind_critic_1/3767355.cp
            dp[Row.at(i), Mask.at(ji)] = coeff * dp[Row.at(i + 1), Mask.at(next_mask(ji, 0))]
```
```cp
// from proto/ports/blind_critic_4/ioi15p6.cp
    for t in 0..n:
        true_dist[V.from_idx(s)!, V.from_idx(t)!] = dist[T.from_idx(t)!]
```
```cp
// from proto/ports/scout_batch_21/dpj.cp
print(fixed(steps[Count(dishes[Pieces(1)]), Count(dishes[Pieces(2)]), Count(dishes[Pieces(3)])], 9))
```

`blind_critic_4/ioi15p6.cp` calls `from_idx(...)!` 40 times in 130 lines. In C++ every one of these
is `a[i]`.

Ideal: two conversions, each with one name, and a rule for when you need neither.

```cp
V(x)            // from the written label, checked: V(1) is the first element of 1..=n
Int(v)          // the written label back; print(v) already prints it
v.to_idx        // the 0-based position, for row-major arithmetic only
```

When the indices of a table are amounts or counters and not identities (`dp[i][mask]`, prefix
tables, digit positions), the port should use a structural range and plain `Int` indices, which the
cheat sheet already allows:

```cp
var dp: [0..=n, 0..m]Z = 0
dp[i, ji] = coeff * dp[i + 1, next_mask(ji, 0)]
```

Verdict: **core**, as a subtraction: keep `V(x)`, `Int(v)`, `v.to_idx`, `V.from_idx(i)`; delete
`.idx`, `.ord`, `.label`, `.nth`, `.at`, `.try`. Then a mechanical migration of `proto/ports/` and a
lint ("three or more conversions on one line: is this domain an amount? use `[0..n]T`").

## 6. Graders and interactors speak raw 0-based `Int`, so every boundary call converts twice

IOI-style signatures pass `Int` and `List<Int>`. The port declares a domain to get checked indexing,
then pays for it at every call in and out:

```cp
// from proto/ports/scout_batch_51/egoi22p3.cp
    let g = Graph.undirected([(V.from_idx(u - 1)!, V.from_idx(v - 1)!) for (u, v) in edges])
```
```cp
// from proto/ports/scout_batch_51/egoi22p3.cp
        let u = V.from_idx(raw - 1)!
        for v in pair_path[u][1..]: MakeMove(v.to_idx + 1)
```
```cp
// from proto/ports/scout_batch_51/egoi22p7.cp
    DescribeDesign([(dsu.find(i).to_idx + 1, i.to_idx + 1) for i in Pin if dsu.find(i) != i])
```
```cp
// from proto/ports/scout_batch_52/ioi22p5.cp
        move_inside(perm[i.to_idx].to_idx)
```
```cp
// from proto/ports/scout_batch_49/ioi22p4.cp
        self.lit.apply(Source.from_idx(l - self.gates)!..=Source.from_idx(r - self.gates)!, true)
```
```cp
// from proto/ports/scout_batch_52/ioi14p1.cp
    ([loc[S(i)] for i in 0..n], [kind[S(i)] for i in 0..n])
```

The same happens on stdin when the input is 1-based and the port wants a 0-based domain
(`+ 1` directly after an index conversion appears 119 times in 90 files):

```cp
// from proto/ports/scout_batch_27/noi07p1.cp
    let a: Int = read()
    let b: Int = read()
    let c: Int = read()
    let (u, v) = (P(a - 1), P(b - 1))
```

Ideal: the signature is where the wire format is declared, exactly as `read()` is typed by the
declared type. A domain parameter converts on the way in and on the way out.

```cp
@grader fn MakeMove(v: V)                              // the judge sees v's label
@grader fn SocialEngineering(n: Int, m: Int, edges: List<(V, V)>):
    domain V = 1..=n
    let g = Graph.undirected(edges)
    ...
    for v in pair_path[u][1..]: MakeMove(v)
```

For a judge that numbers from 0 while the statement numbers from 1 (or the reverse), the domain
says so once: `domain V = 0..n` is the 0-based one, `domain V = 1..=n` the 1-based one, and the
written label is what crosses the boundary. No `base:` argument is needed.

Verdict: **core**, small: `@grader fn` parameters and results of domain type marshal by label, the
same rule `read`/`print` already follow. The forward-reference of `V` from a signature to a
`domain` declared in the body is already used by `scout_batch_37/ioi20p2.cp` ("inferred from the
shape") and needs to be made a stated rule.

## 7. The neighbouring domain (`P` and `P + 1`, a gap and its two ends) is converted by hand

Prefix tables, cut positions and gaps between elements live in a domain one larger or one smaller
than the main one. The cheat sheet has `V.bounds`, `v.l` and `v.r` for exactly this. Only a
handful of the 208 files use them; the rest convert through `Int`:

```cp
// from proto/ports/scout_batch_36/ccc02s4hard.cp
    st.apply(Pos(i.to_idx + 1)..=hi, t)
    stk.push((t, Pos(i.to_idx)))
    let r = Pos(min(n, i.to_idx + m))
    let best = st.fold(Pos(i.to_idx + 1)..=r)!
```
```cp
// from proto/ports/aac5p2.cp
        used[I(next)] = true
        a[I(g.label + 1)] = next
```
```cp
// from proto/ports/scout_batch_51/apio23p2.cp
        for i in pos[v]: st.set(Pre.from_idx(i + 1)!, NP(0, 0, 0, 1))
```

A port that did find the feature shows that the feature has its own tax:

```cp
// from proto/ports/scout_batch_29/noip19p5.cp
var from: [Cut]Cut = [Pos.bounds] c => Pos.bounds.first
```
```cp
// from proto/ports/scout_batch_29/noip19p5.cp
var c = Pos.bounds.last
while c != Pos.bounds.first:
```

Ideal:

```cp
st.apply(i.r..=hi, t)                     // i.r is the cut after i, i.l the cut before
stk.push((t, i.l))
var from: [Cut]Cut = Cut.first            // a `type Cut = Pos.bounds` alias has .first/.last
a[g.r] = next                             // for a gap domain: its left and right element
```

Verdict: **userland** plus a lint. `.l`/`.r` exist; what is missing is that a `type` alias of
`D.bounds` behaves like a domain name (`Cut.first`), that a gap domain declared as
`domain Gap = I.gaps` gets `g.l`/`g.r` returning `I`, and a diagnostic on `D2(x.to_idx + 1)` when
`D2` is `D.bounds` that says "write `x.r`".

## 8. Grid movement that the design solved is still pasted into every file that runs today

The sketches in `scout_batch_31` read well: `p.adj4`, `p.step(d)`, `Dir4.from("UDLR", ch)`. Their
runnable twins do not have any of it. This seven-line helper is pasted, identically, into five
`today/` files:

```cp
// from proto/ports/scout_batch_31/today/ucrpc21g.cp
fn adj4(p: Cell) -> List<Cell>:
    var out: List<Cell> = []
    if let r = p.0 - 1: out.push((r, p.1))
    if let r = p.0 + 1: out.push((r, p.1))
    if let c = p.1 - 1: out.push((p.0, c))
    if let c = p.1 + 1: out.push((p.0, c))
    out
```

One move by a direction letter is eight lines:

```cp
// from proto/ports/scout_batch_31/today/coci25c3p1.cp
    if ch == 'L':
        if let x = c - 1: c = x
    elif ch == 'R':
        if let x = c + 1: c = x
    elif ch == 'U':
        if let x = r - 1: r = x
    elif ch == 'D':
        if let x = r + 1: r = x
```

A table keyed by two cells cannot be indexed by the cells, so every access spreads them:

```cp
// from proto/ports/scout_batch_31/today/dmopc19c4p2.cp
        if dist[a.0, a.1, b.0, b.1] == none:
            dist[a.0, a.1, b.0, b.1] = d + 1
```

Walking a domain backwards is done through arithmetic (six files; `while i >= 0:` with a manual
decrement is in 18 more):

```cp
// from proto/ports/scout_batch_31/today/dmopc21c7p1.cp
for i in 0..n:
    let r = Row(n - 1 - i)
    for j in 0..m:
        let c = Col(m - 1 - j)
```

Even the sketches keep some friction that is not a prototype gap. Manhattan distance between two
cells, an in-bounds test on a point, "this cell and its neighbours", and printing a grid:

```cp
// from proto/ports/scout_batch_31/coci25c3p1.cp
            if abs(p.0 - at.0) + abs(p.1 - at.1) < k: g[p] = ch
```
```cp
// from proto/ports/scout_batch_45/shoi16p2.cp
    if all(p.x in 0..=n and p.y in 0..=m for p in sq): pairs[sorted(sq)] += 1
```
```cp
// from proto/ports/scout_batch_31/today/ucrpc21g.cp
        var targets = adj4(p)
        targets.push(p)
```
```cp
// from proto/ports/scout_batch_31/cco19p2.cp
for r in Row: print(out.row(r))
```

`plan/16-io.md` says `print` prints "grids by row", yet all 11 files that print a `Char` grid loop
over rows, because nobody trusts what the separator will be.

Ideal:

```cp
at = at.step(Dir4.from("UDLR", ch)!) ?? at
if dist[a, b] == none: dist[a, b] = d + 1         // a tuple index fills as many axes as it has
for (r, c) in Cell.rev: ...
if p.dist1(at) < k: g[p] = ch                     // dist1 = Manhattan, dist_inf = Chebyshev
for q in p.adj4_and_self: ...
print(out)                                        // [R, C]Char: one row per line, no spaces
```

Verdict: **toolchain** for `adj4`, `step`, `Dir4`, `D.rev` and tuple-spread indexing (all decided;
`stdlib/grid.cp` already has `step` and `ray`, but `proto/cpc` cannot load it). **Userland** for
`dist1`, `dist_inf`, `adj4_and_self`, and a documented `Show` for `[R, C]Char` (no separator) versus
`[R, C]Int` (spaces).

## 9. Geometry ports do not use `Pt`, because `Pt` cannot be read, printed, ordered or rotated

`std.geometry` has `Pt`, `cross`, `turn`, `ByAngle`. Of the ports that do 2D vector work, most define
their own record instead:

```cp
// from proto/ports/cco16p2.cp
type Vec = (x: Int, y: Int)
```
```cp
// from proto/ports/cco16p2.cp
fn cross(a: Vec, b: Vec) -> Int: a.x * b.y - a.y * b.x
fn upper(v: Vec) -> Bool: v.y > 0 or (v.y == 0 and v.x < 0)
```
```cp
// from proto/ports/cco16p2.cp
    sort(seen, less: (a, b) => cross(a.dir, b.dir) > 0)
```

The reasons show up in the ports that did try `Pt`. It cannot be read:

```cp
// from proto/ports/scout_batch_34/today/dmopc20c7p2.cp
let raw: [Tile](x: Int, y: Int) = read()
```
```cp
// from proto/ports/scout_batch_34/today/dmopc20c7p2.cp
    let c = Pt(raw[t].x, raw[t].y)
```

It does not print as `x y`:

```cp
// from proto/ports/scout_batch_34/dmopc20c7p2.cp
    print("{p.x} {p.y}")
```

It has no order, so sorting points takes a key:

```cp
// from proto/ports/scout_batch_16/ioi07p2.cp
for p in sorted(P, by: p => (pts[p].x, pts[p].y)):
```

A quarter turn is rebuilt from fields although `stdlib/geometry.cp` has `perp`:

```cp
// from proto/ports/scout_batch_45/shoi16p2.cp
    let turn = Pt(-side.y, side.x)
```

Float points are read coordinate by coordinate into a record, and the port brings its own `PI` and
its own Taylor-series `sin_cos`:

```cp
// from proto/ports/scout_batch_21/cmt21peter2.cp
use real.float.{PI, sin_cos, fixed}
```
```cp
// from proto/ports/scout_batch_21/cmt21peter2.cp
var at: [P](x: Float, y: Float) = (x: 0.0, y: 0.0)
```
```cp
// from proto/ports/scout_batch_21/sac22cc4jp3.cp
    let g = gap(Float(aim), atan2(y, x) * 180 / PI)
```

Ideal:

```cp
let hares: [Hare](at: Pt, w: Int) = read()
sort(mut seen, by: s => ByAngle(s.dir))           // or by: s => s.dir.half_angle for half-plane sweeps
for p in sorted(P, by: p => pts[p]): ...          // Pt orders by (x, y)
print(p)                                          // "x y"
let turn = side.perp
let deg = p.angle_deg                             // atan2 in degrees, 0..360; PI is in the prelude
if p in Box(Pt(0, 0), Pt(n, m)): ...
```

Verdict: **userland**. `Pt<T>` should derive `Read`, `Show`, `Eq`, `Ord` and `Hash` like any
two-field struct (the cheat sheet says derivation is automatic; make it true for `Pt` in every
tool), and `std.geometry` gains `angle_deg`, `Box`, `reflect`, `dist`, `PI`. `sin`, `cos` and
`atan2` are intrinsics on the cheat sheet; `proto/cpi` needs them so that nobody writes a Taylor
series again.

## 10. Mixed integer and float arithmetic needs a cast on every operand, and there are two names for the type

The type is `Float` in 19 files and `F64` in 13. The cheat sheet says `F64`. The rule "an `Int`
variable needs `F64(x)`" produces lines where the casts outnumber the operators:

```cp
// from proto/ports/scout_batch_23/noi07p1.cp
                            total += Float(ways[s, v] * ways[v, t]) / Float(ways[s, t])
```
```cp
// from proto/ports/scout_batch_36/cco19p4.cp
    if k == 4: F64(d) * F64(d)
```
```cp
// from proto/ports/scout_batch_22/noi09p2.cp
fn through(j: Int, i: Int) -> F64 = best[j] + F64(line(j, i)) ** F64(p)
```
```cp
// from proto/ports/scout_batch_14/noi05p4.cp
    1.0 + total / Float(deg + 1)
```

In C++ the first line is `total += (double)a * b / c;`. Optional floats have no ordering, so a
Li Chao tree needs a five-line comparison, and the runnable twin falls back to a `-1e300` sentinel:

```cp
// from proto/ports/scout_batch_36/cco19p4.cp
fn beats(a: ?F64, b: ?F64) -> Bool:
    if let x = a:
        if let y = b: x > y else: true
    else: false
```

When a port wants to stay exact, it writes its own fraction comparison by cross-multiplication.
Three files do, each differently:

```cp
// from proto/ports/scout_batch_16/ccc00s5.cp
fn le(a: Frac, b: Frac) -> Bool = a.num * b.den <= b.num * a.den
```
```cp
// from proto/ports/scout_batch_29/ioi09p2.cp
    a.points * ask[a.setter].s * ask[b.setter].q < b.points * ask[b.setter].s * ask[a.setter].q
```

Ideal:

```cp
total += ways[s, v] * ways[v, t] / F64(ways[s, t])     // one cast makes the expression F64
fn through(j: Int, i: Int) -> F64 = best[j] + F64(line(j, i)) ** p
if c.eval(x) max> best: ...                            // or: best max= c.eval(x), see pattern 15
sort(mut by_rate, by: r => Frac(r.s, r.q))             // exact, I128 cross-multiplication inside
```

Verdict: **core** for one relaxation: in a binary arithmetic operation where one side is `F64`, an
`Int` on the other side promotes (exact below 2^53, with the existing local trap above it). That
removes a rule from the cheat sheet instead of adding one, and keeps `Int / Int` integral. **Core**
housekeeping: `Float` is deleted as a name. **Userland** for `Frac` in `std.math` with `Ord`, `+`,
`-`, `*` and `fixed(k)`.

## 11. `Mod<P>` arithmetic wraps every literal, and runtime moduli fall back to method calls

`Z(...)` around a literal or a plain `Int` expression appears 216 times in 60 files. The cheat sheet
says `Int` reduces one way into `Mod<P>`, so almost all of them should not exist:

```cp
// from proto/ports/scout_batch_11/coci21c2p4.cp
            if w == Z(0): continue
            if k + 1 <= l: next[j + 1][k + 1] += w * Z(j + 1)
```
```cp
// from proto/ports/scout_batch_29/noip19p4.cp
for i in Method: all = all * Z(total[i] + 1)
print(all - Z(1) - bad)
```
```cp
// from proto/ports/scout_batch_23/noimps22p4.cp
var pow2 = List.filled(n + 1, Z(1))
for i in 1..=n: pow2[i] = pow2[i - 1] * Z(2)
```

With a modulus read at run time, one sketch uses a `ModR` object and loses operators entirely, and
its runnable twin writes `% p` by hand with its own `mod_pow`:

```cp
// from proto/ports/scout_batch_14/noi15p3.cp
        cur = z.mul(cur, z.sub(z.add(z.pow(2, acnt), z.pow(2, bcnt)), 1))
```
```cp
// from proto/ports/scout_batch_14/today/noi15p3.cp
            let factor = (mod_pow(2, acnt) + mod_pow(2, bcnt) - 1) % p
            cur_way = (cur_way * factor) % p
```

Raw `% MOD` arithmetic on `Int` survives in 17 files (69 lines), including next to a `Mod` type:

```cp
// from proto/ports/scout_batch_29/noip19p4.cp
let total = [Method] i => sum(a[i, j] for j in Food) % 998244353
```

Inclusion-exclusion signs are an `if`/`else` each time (also `scout_batch_29/shoi16p4.cp`), and
there is `sum(it)` but no `product(it)`:

```cp
// from proto/ports/scout_batch_23/noimps22p4.cp
        if popcount(sub) % 2 == 0: ans += term
        else: ans -= term
```
```cp
// from proto/ports/scout_batch_49/ioi22p4.cp
            for v in kids[u]: all[u] *= all[v]
```

Ideal:

```cp
type Z = Mod<p>                                   // p read at run time: same type, same operators
cur *= Z(2) ** acnt + Z(2) ** bcnt - 1
if w == 0: continue
next[j + 1][k + 1] += w * (j + 1)
let pow2 = Z(2).powers(n)                         // [0..=n]Z
ans += term * sign(sub.len)                       // sign(k) = 1 if k is even, else -1
all[u] = product(all[v] for v in kids[u])
let total = [Method] i => sum(Z(a[i, j]) for j in Food)
```

Verdict: **toolchain** for literal and `Int` lifting and for `Mod<p>` over a runtime `let` (both are
on the cheat sheet; `proto/cpi` rejects `Mod + Int` and has no runtime modulus, which is why `ModR`
and `Z(1)` exist). **Userland** for `powers`, `sign(k)`, `product(it)`. Delete `ModR` from every
document.

## 12. Numbers wider than 63 bits are rebuilt from two `Int` limbs

Four ports implement their own wide integer. The reasons: an input up to 2^64 - 1 cannot be read
into an `Int`, a 36-digit answer cannot be printed, or a count overflows.

```cp
// from proto/ports/scout_batch_22/bits.cp
pub struct U64(hi: Int, lo: Int):
    pub fn parse(s: Str) -> U64:
        var (hi, lo) = (0, 0)
        for c in s:
            lo = lo * 10 + (c - '0')
            hi = (hi * 10 + lo / half) & low
            lo = lo & low
        U64(hi, lo)
```
```cp
// from proto/ports/scout_batch_29/noip19p5.cp
    total = total.plus(Wide.square(s[c] - s[from[c]]))
```
```cp
// from proto/ports/blind_critic_4/ioi11p6.cp
                s_lo += dp_lo[rp, ak]
                s_hi += dp_hi[rp, ak] + s_lo / BASE
                s_lo = s_lo % BASE
```
```cp
// from proto/ports/scout_batch_25/today/noip19p1.cp
let M32 = 4294967295         // not `(1 << 32) - 1`: emitted as a 32-bit shift today
```

The last line is a separate defect: `1 << 32` miscompiles in `proto/cpc`, so ports define
`fn bit(i: Int) -> Int: 1 << i` (`scout_batch_25/today/ioi20p4.cp`,
`scout_batch_51/today/cco03p4.cp`) to dodge it.

0.2.2 has `U64` and `I128` as numeric types. None of these four ports could use them.

Ideal:

```cp
let (n, k): (1..=64, U64) = read()
print("{k ^ (k >> 1):0{n}b}")
var total: I128 = 0
total += I128(s[c] - s[from[c]]) ** 2
print(total)
```

Verdict: **toolchain**. `U64` and `I128` need `Read`, `Show`, shifts and format flags in `cpi`, `cpc`
and the Rust back ends, and the 32-bit shift bug needs fixing before anything else in this list,
because it produces silent wrong answers. No new library type is needed: every wide value in the
four ports fits in 128 bits.

## 13. A query or command with a kind tag is read field by field and dispatched with `if`

44 files read a tag and branch on it:

```cp
// from proto/ports/scout_batch_39/dmpg18s5.cp
    let kind: 1..=2 = read()
    if kind == 1:
        let (l, r, x): (1..=n, 1..=n, 1..=MAXV) = read()
        let (li, ri) = (l - 1, r - 1)
```
```cp
// from proto/ports/scout_batch_29/sdoi17r1p6.cp
    let (kind, l, r): (Int, Pos, Pos) = read()
    if kind == 1:
```
```cp
// from proto/ports/scout_batch_22/noi14p1.cp
    let (gate, t): (Str, Int) = read()
    if gate == "AND": (from_zeros, from_ones) = (from_zeros & t, from_ones & t)
    elif gate == "OR": (from_zeros, from_ones) = (from_zeros | t, from_ones | t)
    else: (from_zeros, from_ones) = (from_zeros ^ t, from_ones ^ t)
```
```cp
// from proto/ports/scout_batch_20/btoi17p5.cp
    let sign: Char = read()
    let row: Int = read()
    let col: Int = read()
    cells.push(Fixed(plus: sign == '+', row: row, col: col))
```

The cheat sheet has the answer for single characters (`@tags('.', '*', ...)` on an enum), one
top-level port uses it with strings (`@tags("R", "C")` in `ccc21s2.cp`), and nobody uses it with
numbers because it is not stated that numbers work.

Ideal:

```cp
@tags(1, 2)
enum Query: Count(l: I, r: I, x: 1..=MAXV) | Set(i: I, x: 1..=MAXV)
for q in read::<[0..q]Query>():
    match q:
        Count(l, r, x) => ...
        Set(i, x) => ...

@tags("AND", "OR", "XOR")
enum Gate = And | Or | Xor
let gates: [0..n](op: Gate, t: Int) = read()
```

Verdict: **userland**: the derived `Read` for an enum takes integer, character and string tags.
State it on the cheat sheet in one line, and add an example where a payload-less enum sits inside a
record, as in the second case.

## 14. The local judge is written twice, once as `ask(...)` calls and once as a `recv` loop

Ten files have an `@interactor`. Each re-parses the wire protocol the solution already described
through its `ask` calls: a string tag, an `if`/`elif` chain, one `recv` per arm, a manual query
counter, and `assert false` for the default. This one is 26 lines:

```cp
// from proto/ports/scout_batch_34/dmopc20c6p6.cp
    while true:
        let op: Str = sol.recv()
        if op == "E":
            let x: Jug = sol.recv()
            cur[x] = 0
            queries += 1
        elif op == "F":
            let x: Jug = sol.recv()
            cur[x] = hidden[x]
            queries += 1
```

The solution side repeats the query budget in every call (`limit: 4 * n` three times in that file),
and the judge counts again and checks once at the end:

```cp
// from proto/ports/scout_batch_34/dmopc20c6p6.cp
            assert queries <= 4 * n
```

The judge also re-implements the problem's simulation with raw integers instead of sharing the
solution's types:

```cp
// from proto/ports/scout_batch_19/egoi23p3.cp
                if nr in 0..h and nc in 0..w and (nr != br or nc != bc):
```

When the reply must come before the next question (the judge speaks first, then each answer is also
the next question), the first round is duplicated:

```cp
// from proto/ports/scout_batch_19/today/ioi14p3.cp
    if step_i == 0:
        let e: (V, V) = ask("READY")
        reply = has_edge(mut par, mut comp_sz, mut edge_cnt, e.0, e.1)
    else:
        let e: (V, V) = ask(reply)
        reply = has_edge(mut par, mut comp_sz, mut edge_cnt, e.0, e.1)
```

Worst, the `today/` version of an interactive problem often is not interactive at all: it reads the
hidden answer from stdin and calls a local function named `ask`
(`scout_batch_44/today/dmopc21c2p3.cp`, `scout_batch_52/today/noi02p2.cp`,
`scout_batch_52/today/ioi20p5.cp`). Those ports prove the algorithm and nothing about the protocol.

Ideal: the protocol is one tagged enum (pattern 13), shared by both sides. The solution's queries
are thin functions over `ask`; the judge is a `match` on the same enum; the budget is declared once.

```cp
@tags("E", "F", "D", "A")
enum Msg: Empty(x: Jug) | Fill(x: Jug) | Pour(x: Jug, y: Jug) | Answer(cap: [Jug]Int)

@interactor(limit: 4 * n)
fn judge(sol: Interactor):
    ...
    for msg in sol.recv_all::<Msg>():
        match msg:
            Empty(x) => cur[x] = 0
            Fill(x) => cur[x] = hidden[x]
            Pour(x, y) => sol.send(pour(x, y))
            Answer(cap) => return assert cap == hidden
```

Verdict: **userland** for `recv_all::<T>()` and the enum `Read`; **core**, small, for
`@interactor(limit: K)` as the one place the total budget lives (the per-call `limit:` stays for
per-kind budgets). **Toolchain**: `proto/interact.py` must run these, so that `today/` ports stop
faking the judge.

## 15. "Keep the best so far" is a four-clause `if`, although `min=` already skips `none`

`x == none or ...` appears 49 times in 38 files. Almost every one is a hand-written `min=`:

```cp
// from proto/ports/scout_batch_31/today/noi10p3.cp
            if best == none or v < best!: best = v
```
```cp
// from proto/ports/scout_batch_31/today/dmopc21c6p4.cp
            if best == none or x < best!.0: best = (x, s)
```
```cp
// from proto/ports/scout_batch_21/sac22cc4jp3.cp
    if best == none or g < best!.gap: best = (gap: g, id: id)
```
```cp
// from proto/ports/blind_critic_5/noiwc22p3.cp
        if best_score == none or score < (best_score ?? 0) or (score == (best_score ?? 0) and a < best_word):
```

The second shape is the relaxation step of every hand-written shortest path, where the update has
to be followed by a push:

```cp
// from proto/ports/scout_batch_31/today/ucrpc21f.cp
        if dist[q] == none or nd < dist[q]!:
            dist[q] = nd
            pq.push((nd, q))
```

and the non-optional one, by hand, in older ports:

```cp
// from proto/ports/blind_critic_4/ioi17p6.cp
        if cur < mn:
            mn = cur
        if cur > mx:
            mx = cur
```

Ideal: the first group is already expressible and needs only a lint. The second needs `min=` to
say whether it changed anything, which is the `chmin` every C++ contestant has in a template.

```cp
best min= v
best min= (x, s)                          // tuples compare lexicographically
best min= (gap: g, id: id)
if dist[q] min= nd: pq.push((nd, q))      // `x min= y` as a condition: true when x changed
```

Verdict: **core**, small: `x min= y` and `x max= y` have type `Bool` when used as an expression. It
extends an existing operator instead of adding a function, and it removes the reason for 40 hand
queue loops to spell out the comparison. Plus a lint for `a == none or b < a!`.

## 16. Output that is "a count, then the items" or "one per line" is assembled manually

57 files print a length on its own line. The list that follows is printed in at least five ways:

```cp
// from proto/ports/scout_batch_45/iati21a23.cp
print(shots.len)
print(shots)
```
```cp
// from proto/ports/scout_batch_25/today/ioi18p4.cp
print(X.len, X)
```
```cp
// from proto/ports/scout_batch_45/neerc10k.cp
print(k)
for v in V: print(colour[v]!)
```
```cp
// from proto/ports/scout_batch_29/ioi09p2.cp
            print(hired.len)
            while not hired.is_empty: print(hired.pop()!.1)
```
```cp
// from proto/ports/blind_critic_5/ioi19p5.cp
    var line: List<Int> = [gate_ops[i], gate_args[i].len]
    for a in gate_args[i]:
        line.push(a)
    print(line)
```

A space-separated line is joined by hand where `print(list)` was not trusted:

```cp
// from proto/ports/scout_batch_44/today/dmopc21c5p3.cp
fn show(xs: List<Int>):
    var line = ""
    for x in xs:
        if line.len > 0: line = line + " "
        line = line + "{x}"
    print(line)
```

Tuples and records are taken apart to print them:

```cp
// from proto/ports/scout_batch_19/egoi23p3.cp
print("!", cands[0].0, cands[0].1)
```
```cp
// from proto/ports/yac6p3.cp
    print("!", p.x, p.y, q.x, q.y)
```
```cp
// from proto/ports/scout_batch_25/noi20p4.cp
        print(*(x for (i, g) in dish for x in (i + 1, g)))
```

A 2D table is printed by building each row:

```cp
// from proto/ports/scout_batch_25/today/ioi20p3.cp
for c in Colour:
    var row: List<Int> = []
    for t in Ticket:
        row.push(round[c, t])
    print(row)
```

Ideal: `print` flattens tuples, records and lists into space-separated fields on one line (stated
once, true everywhere), 2D arrays print one row per line, and one named argument covers "one per
line".

```cp
print(shots.len); print(shots)
print(k); print(colour, sep: "\n")
print(gate_ops[i], gate_args[i].len, gate_args[i])
print("!", cands[0])
print("!", p, q)
print(dish)                               // a list of pairs prints i g i g ...
print(round)                              // [Colour, Ticket]Int: one row per line
```

Verdict: **userland**: these are `Show` rules for tuples, lists and 2D arrays plus a `sep:` argument
on `print`. They need to be written down in `plan/16-io.md` and on the cheat sheet, because the
evidence is that port authors did not know what `print(xs)` would do and wrote loops to be safe.

## 17. Characters are mapped to small numbers with `if` chains and arithmetic on `'0'` and `'a'`

Character arithmetic against `'0'`, `'a'` or `'A'` is in 29 files. Larger alphabets get a function:

```cp
// from proto/ports/scout_batch_22/ioi21p4.cp
fn base(c: Char) -> Base = Base(if c == 'A': 0 elif c == 'C': 1 else: 2)
```
```cp
// from proto/ports/scout_batch_12/cco02p3.cp
var face = 0
if facing == 'E': face = 1
if facing == 'S': face = 2
if facing == 'W': face = 3
```
```cp
// from proto/ports/scout_batch_31/today/ucrpc21d.cp
fn wall(p: Cell) -> Bool: g[p] == '+' or g[p] == '-' or g[p] == '|'
```
```cp
// from proto/ports/scout_batch_43/etsr23r4p1.cp
        print(Str([if cur[v] == 1: '(' else: ')' for v in V]))
```
```cp
// from proto/ports/blind_critic_5/noiwc22p3.cp
        code = code * 2 + (if gold_str[i] == '1': 1 else: 0)
```

Prefix tests are redefined in two string ports with the same body:

```cp
// from proto/ports/scout_batch_22/cco19p6.cp
fn starts_with(s: Str, head: Str) -> Bool = head.len <= s.len and s[0..head.len] == head
```

Ideal: an enum with `@tags` is the alphabet, and it reads from and prints to a string directly.

```cp
@tags('A', 'C', 'T')
enum Base = A | C | T
let (a, b): ([0..n]Base, [0..n]Base) = read()      // a string of n letters reads as n Bases

var face: Dir4 = read(tags: "NESW")
fn wall(p: Cell) -> Bool: g[p] in "+-|"
print(Str("()"[Int(cur[v] != 1)] for v in V))
let code = Int.from_bits(gold_str)                 // "10110" -> 22
```

Verdict: **userland**: derived `Read`/`Show` for `[D]Enum` from an unbroken string when every tag is
one character, `Char in Str`, `Int.from_bits(Str)`, and `starts_with` actually present in the
prelude (it is on the cheat sheet, and two ports had to write it).

## 18. Bit tests, bit building and submask loops are spelled with shifts and masks

`(x >> i) & 1` tests appear 35 times in 21 files, usually with a comparison bolted on:

```cp
// from proto/ports/scout_batch_48/egoi22p8.cp
            if mask >> j & 1 == 1:
```
```cp
// from proto/ports/scout_batch_52/ioi20p5.cp
        for i in 0..k: known[((mask >> i) & 1) == 1].push(vars[i])
```
```cp
// from proto/ports/scout_batch_19/today/ioi16p5.cp
        if ((a / (1 << bit)) % 2) == 1:
```

Submask enumeration and subset tests are the C idioms, typed out:

```cp
// from proto/ports/scout_batch_23/noimps22p4.cp
        if sub == 0: break
        sub = (sub - 1) & sm
```
```cp
// from proto/ports/scout_batch_51/today/cco03p4.cp
            if (s & b) == 0 and (need[V(v)] & s) == need[V(v)]:
```

Digits in another base are divided out by hand:

```cp
// from proto/ports/scout_batch_14/today/noi15p3.cp
                    let digit = (m3 / pow3[prime_id[c] - 1]) % 3
```
```cp
// from proto/ports/scout_batch_48/egoi22p8.cp
    return rank(x / 128 / 128 / 128, x / 128 / 128 % 128, x / 128 % 128, x % 128)
```

and `bitrev`, `top_bit`, `popcnt` and "smallest power of two at least n" each have a hand loop in at
least one port (`scout_batch_25/today/ioi18p4.cp`, `scout_batch_25/today/ioi20p4.cp`,
`blind_critic_1/3767355.cp`).

Ideal: `Subset<D>` is the answer when the bits are a set, and it already has `v in s`,
`for sub in mask.subsets` and set operators. For bits of a plain number, three methods:

```cp
if j in mask: ...                                  // mask: Subset<Seg>
for sub in sm.subsets: ...
if v not in s and need[v] <= s: ...                // <= is subset-of
known[mask.bit(i)].push(vars[i])                   // Int.bit(i) -> Bool
let (a, b, c, d) = x.digits(128, 4)                // most significant first
let leaves = n.next_pow2
```

Verdict: **userland**: `Int.bit(i)`, `Int.digits(base, k)`, `Int.next_pow2`, `bit_reverse(x, k)`,
and `<=` on `Subset<D>`. `popcount`, `bit_len` and `mask.subsets` exist and need a lint that
recognises the hand loops.

## 19. Cyclic neighbours, wrap-around ranges and "previous element or default" are open-coded

`(i + 1) % n` used as an index appears in 14 files; `xs[xs.len - 1]` instead of `xs.last` appears 79
times in 58 files.

```cp
// from proto/ports/scout_batch_34/dmopc20c7p2.cp
        let v = c + shape[(j + 1) % m]
```
```cp
// from proto/ports/scout_batch_37/ioi20p2.cp
                let (a, b) = (cycle[k], cycle[(k + 1) % cycle.len])
```
```cp
// from proto/ports/scout_batch_52/today/noi02p2.cp
        dir_l[p] = ask_ring(p, (p + n - 1) % n)
```
```cp
// from proto/ports/scout_batch_13/ioi05p4.cp
    let gap = max(b - a for (a, b) in (shifts ++ [shifts[0] + n]).adjacent)
```

A range that wraps around the end of a circular array is split by hand:

```cp
// from proto/ports/blind_critic_4/ioi20p1_draft.cp
            if let lo = u - (k - 1):
                st.apply(lo..=u, 1)
            else:
                st.apply(..=u, 1)
                st.apply(u.cyc(-(k - 1)).., 1)
```

Out-of-range reads that should give a default are a conditional each:

```cp
// from proto/ports/scout_batch_25/ioi20p4.cp
        let have = carry + (if i < a.len: a[i] else: 0)
```
```cp
// from proto/ports/scout_batch_34/dmopc21c2p5.cp
                    let prev_d = if let pd = d - 1: cur[pd, i] else: Z(0)
```

Ideal:

```cp
for (a, b) in cycle.cyclic_adjacent: ...           // n pairs, the last one wraps
let gap = max(b - a for (a, b) in shifts.cyclic_adjacent(period: n))
for r in u.cyc(-(k - 1)).cyc_to(u): st.apply(r, 1) // one or two plain ranges
let have = carry + (a.get(i) ?? 0)                 // get -> ?T on every indexed container
let prev_d = cur.get((d - 1, i)) ?? 0              // a none index gives none
```

Verdict: **userland**: `cyclic_adjacent`, `v.cyc_to(w)` yielding the one or two ranges, and a
uniform `.get(i) -> ?T` that accepts an `Int`, a `?D` or a tuple containing one. Lint
`xs[xs.len - 1]` to `xs.last!`.

## 20. The `today/` tax: decided features that do not run, so ports fall back to C-style code

This is not one pattern but the sum of the "toolchain" verdicts above, and it deserves its own
entry because of its size. For the 47 sketch and `today/` pairs among the 208 files read, the runnable twins total 1,979
code lines against 1,344 for the sketches (47% longer), and the extra lines are of this kind:

A queue is a list and a head index (40 times in 37 files), although `Deque` is in the prelude:

```cp
// from proto/ports/scout_batch_14/noi05p4.cp
    var q: List<Node> = [target]
    var head = 0
    while head < q.len:
        let u = q[head]
        head += 1
```

Binary search over a sorted list is written out (the `let mid = (lo + hi ...` line is in 58 files),
although `first_ge` and `bsearch_first` exist:

```cp
// from proto/ports/scout_batch_26/ioi09p7.cp
    while lo < hi:
        let mid = (lo + hi) / 2
        if spots[x][mid] < p: lo = mid + 1
        else: hi = mid
```

Test cases are a `for` loop over a count (113 files) instead of `@multitest`, and 23 files wrap the
whole program in `fn main_solve()` to get an early `return`:

```cp
// from proto/ports/scout_batch_21/sdoi17r2p1.cp
let tests: Int = read()
for _ in 0..tests:
```

Descending order is a negated key, because a max-heap or `by: desc` was not available:

```cp
// from proto/ports/scout_batch_25/today/ioi09p3.cp
    ranking.push((-score, -tasks, c.to_idx + 1))
sort(mut ranking)
for r in 0..N:
    if ranking[r].2 == P:
        print(-ranking[r].0, r + 1)
```

Randomness is a hand-rolled generator in six files, although `rand` and `shuffle` are prelude names:

```cp
// from proto/ports/scout_batch_47/today/cco07p5.cp
    rng_state = (rng_state * 1_103_515_245 + 12_345) % 2_147_483_648
```

Coordinate compression, `next_permutation`, a swap through a temporary (20 files), a list copy by
loop, and memo tables as parallel lists all recur the same way:

```cp
// from proto/ports/scout_batch_52/today/ioi20p5.cp
    let t = a[i]
    a[i] = a[j]
    a[j] = t
```

Stateful graders copy the whole state out and back in to change one cell:

```cp
// from proto/ports/scout_batch_52/ioi13p3.cp
    var w = state!
    w.h[p][q] = cost
    w.rebuild_leaf(p / B)
    state = w
```

Ideal: nothing new. `Deque`, `xs.first_ge(p)`, `@multitest`, top-level `return`, `by: desc`,
`rand(lo..hi)`, `shuffle`, `compress`, `next_permutation`, `swap(mut a[i], mut a[j])`, `@memo`, and
`state!.h[p][q] = cost` as a place expression are all on the cheat sheet.

Verdict: **toolchain**, with one **core** clarification: `x!` on a `var` of type `?T` is a place, so
`state!.h[p][q] = cost` mutates in place without a copy. The practical recommendation is to stop
accepting `today/` ports that work around a decided feature: each workaround should instead become
a failing test in the prototype, so that the gap is closed once and not paid for in every port.

## What to do first

Ranked by contest minutes saved per unit of work:

1. Fix the things that silently produce wrong output: the dropped `{x:.kf}` format spec in
   `proto/cpi`, bare `print(F64)`, and the 32-bit `1 << k` in `proto/cpc` (patterns 1 and 12).
2. Collapse the spellings. One float format (`"{x:.3f}"`), one float type name (`F64`), one
   explicit-type read (`read::<T>()`), two index conversions (`V(x)` / `Int(v)` for labels,
   `to_idx` / `from_idx` for positions). This is deletion work in `plan/16-io.md`, the cheat sheet
   and about 900 call sites, and it adds nothing to the language (patterns 1, 4, 5, 10).
3. Land the four small core rules, each of which generalises something that already exists:
   expected-type flow into `read`/`ask` (4), label marshalling for `@grader` signatures (6),
   `Int` promotion next to an `F64` (10), and `min=`/`max=` as a `Bool` expression (15).
4. Add the userland batch in one pass, because every item is a few lines of `stdlib/`:
   `fixed(num, den, k)`, `Frac`, `Char.digit`, `Bool` read/print with `yes:`/`no:`, `Pt` derives
   and `Box`/`angle_deg`/`reflect`, `dist1`/`dist_inf`, `powers`/`sign`/`product`, integer and
   string `@tags`, `recv_all`, `print(.., sep:)`, `Int.bit`/`digits`/`next_pow2`,
   `cyclic_adjacent`, `.get` (2, 3, 8, 9, 10, 11, 13, 14, 16, 17, 18, 19).
5. Close the prototype gaps behind pattern 20 and reject new `today/` workarounds for decided
   features.

Concept budget check: the proposals add no keyword and no decorator. They add one decorator
argument (`@interactor(limit:)`), two `print`/`read` named arguments (`yes:`/`no:`, `sep:`) modelled
on the existing `none:`, and they remove six index-conversion spellings, two float-print spellings,
one type name (`Float`), one library type (`ModR`) and one output function (`write`).

## Files read

The 208 files can be listed again from `proto/ports/` with:

```sh
cd proto/ports
(grep -rlE 'cross\(|Pt\(|hypot|sqrt\(|atan|F64|Float' --include='*.cp' .
 grep -rlE '@interactor|@grader|ask\(|@checker' --include='*.cp' .
 grep -rlE '\{[a-z_.()\[\] ]*:[^}]*\}|pad|fixed' --include='*.cp' .
 find scout_batch_22 scout_batch_25 scout_batch_29 scout_batch_31 -name '*.cp' | sed 's|^|./|') | sort -u
```

The pattern counts in the text were taken with `grep -rE` over every `.cp` file under
`proto/ports/` at commit `4436273`.
