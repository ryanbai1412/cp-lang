# Friction backlog, batch 16: messy fallbacks and promises a type cannot hold

A Stage-1 scout pass over ten submissions chosen to be awkward: hand-scanned
text, exact geometry, a multi-phase grid simulation, a randomized
completion, planar maps, and two interactive problems. The ports are in
[`proto/ports/scout_batch_16/`](../proto/ports/scout_batch_16/): noip07p2,
ccc00s5, cco05p4, coci21c6p5, ccc19s3, noi03p1 and ioi00p4 run today;
ioi18p1, ioi13p4 and ioi07p2 are ideal sketches.

The pass asks two questions. When none of the sugar fits, is plain CP Lang
still pleasant (items `M1`...)? And when an input is only valid as a whole,
so that no field type can say it, is `cpl stress` still one or two lines
(items `Q1`...)? The letters do not collide with `F`, `G`, `B`, `K`, `S`,
`D`, `H` or `T` of earlier batches.

What was checked: the seven runnable ports run on the reference interpreter
(`proto/cp.py lib-run`, at commit `f5030cd`) and agree with their originals
on 300 small random inputs each
(`python3 proto/ports/scout_batch_16/check.py 300`); ccc19s3 has many right
answers, so both outputs are validated against the statement instead. Every
"today" snippet below is a line a port really contains, and every error
quoted came from a probe on that interpreter. The "wanted" snippets do not
run. The three sketches were never run, and nothing was measured at full
input size.

The short answer to the first question is yes, with a caveat. No port had
to fight the language: records, lists, loops, `?T` with `let ... else`, and
small helper fns carried all seven, and three got shorter by rethinking
(cco05p4 lost its sieve, ccc19s3 its duplicated row and column code,
ioi00p4 its edge matrix). What hurt was a ring of small missing things
round the core: formatting, joining, scanning, reading a plain list.
Each cost two to five lines, and together they are most of the distance
between these ports and how they should read.

The short answer to the second is not yet. The design has `where` for a
relation between two values and `read(gen: f)` for a custom generator, and
that is enough for none of the six promises this batch met. Q1 to Q5 are
the missing pieces.

## Suggested order for a Stage-2 sprint

1. Q1 and Q2 together: `assume` and a `gen` that is tested against it.
   They are small, general, and five of the ten ports want them.
2. M1 and M2: formatting and joining. Every port that prints more than one
   number per line pays today.
3. M4: `read` for a plain list. Three ports declare a domain only to
   read one.
4. Q3 and Q4: the stdlib generators and generate-from-a-witness.
5. M3: interpreter gaps against the tour, a few lines each.
6. Q5, with the `protocol` work it depends on.
7. The rest, each seen once.

## M1. Formatting a number (2 of 10: ccc00s5, ccc19s3)

ccc00s5 must print coordinates with two decimals. An interpolation takes no
format: `"{cents:02}"` is accepted and prints `0`, not `00`, with no error.
So the port carries a helper that pads by hand:

```
// today (ccc00s5.cp)
fn shown(v: Int) -> Str:
    let cents = v % SCALE / 10
    "{v / SCALE}.{cents / 10}{cents % 10}"

print("The sheep at ({shown(px)}, {shown(py)}) might be eaten.")
```

The same port reads its coordinates as `Float` and scales them
(`Int(x * SCALE)`), which is the original's trick and is only safe because
of the extra factor of ten.

```
// wanted
let sheep: List<(x: Fixed<2>, y: Fixed<2>)> = read(n)
print("The sheep at ({p.x}, {p.y}) might be eaten.")
print("{cents:02} {ratio:.6}")
```

Fix, stdlib and a small core rule. (a) An interpolation hole takes a format
after a colon: width, zero padding, and digits after the point. A format the
value does not support is a compile error, never ignored. (b)
`Fixed<D>` in the stdlib: an `Int` scaled by `10^D` that reads `12.5` or
`12.50` exactly, prints with `D` digits, and has the arithmetic of `Int`. It
is an ordinary struct with `Read` and `Show`, so nothing is added to the
core. Its generator draws the scaled integer, so ties and equal distances
come up in stress, which they almost never do with a `Float` generator.

## M2. Printing a list on one line (3 of 10: coci21c6p5, ccc19s3, cco05p4)

`plan/16-io.md` says `print(xs)` prints a list space-separated. The
interpreter prints `[0, 1]`. There is no `join` either (`"".join(out)`:
"has no method `join`"), so coci21c6p5 builds its answer line by hand:

```
// today (coci21c6p5.cp)
var line = Str()
for (_, y) in pairs:
    if line.len > 0: line.push(' ')
    line.push(if kept[y]: '1' else: '0')
print(line)
```

```
// wanted
print(if kept[y]: 1 else: 0 for (_, y) in pairs)
```

Fix, interpreter and stdlib: make `print` of a list or a generator do what
the plan says, and add `xs.join(sep)` for the cases that are not one line
of output. ccc19s3 prints a row as `"{row[0]} {row[1]} {row[2]}"` for the
same reason.

## M3. `count` and `enumerate` (2 of 10: ccc19s3, coci21c6p5)

The tour uses both. The interpreter has neither ("unknown name
`count`"). The ports fall back to a filtered `sum` and an index:

```
// today
if sum(1 for x in v if x == none) != 1: continue        // ccc19s3.cp

for i in 0..n:                                          // coci21c6p5.cp
    let (x, y) = pairs[i]
```

```
// wanted
if count(x == none for x in v) != 1: continue
for (i, (x, y)) in enumerate(pairs):
```

Fix, stdlib: generic fns over any iterable, written in CP Lang, together
with `first(...) -> ?T`, which composes with `let ... else`. No core
change.

## M4. `read()` only works as a typed `let` (3 of 10: ioi00p4, ccc19s3, coci21c6p5)

`read()` is recognized only as the whole right side of a `let` with a type.
`[read() for _ in 0..k]` fails with "unknown name `read`", and an array
type needs a named domain: `let a: [0..k]Int = read()` fails with "`Int`
is not an index type". So a list whose length was just read costs a
throwaway domain, even when nothing else ever indexes by it:

```
// today (ioi00p4.cp)
domain Member = 0..k
let homes: [Member]Town = read()
...
    let len: Int = read()
    domain Side = 0..len
    let border: [Side]Town = read()
```

`Side` earned its keep (`border[s.cyc(1)]` is the next corner round the
region, with no `% len`). `Member` did not. And ccc19s3's token that is
either `X` or a number has to go through `Str`:

```
// today (ccc19s3.cp)
fn read_cell() -> ?Int:
    let w: Str = read()
    if w == "X": none else: Int(w)
```

```
// wanted
let homes: List<Town> = read(k)
let border: List<Town> = read()             // "count, then that many": the usual length prefix
let given: [3, 3]?Int = read(none: "X")
```

Fix, core `Read` rules, no new construct: `read(k)` for a `List<T>` of a
known length, `read()` for a `List<T>` meaning length-prefixed, and a
`none:` spelling for `?T`. All three are ordinary arguments to `Read`
impls. The generator follows from the type in each case, which the
hand-written `read_cell` loses.

## M5. Characters as small numbers (2 of 10: noip07p2, noi03p1)

Text problems move between a character and its number constantly. Today
that is `.ord` arithmetic on both sides:

```
// today
if p1 == 2 and c.is_lower: return Char(c.ord - 'a'.ord + 'A'.ord)      // noip07p2.cp
var fill = [shown(Char(c)) for c in s[i - 1].ord + 1..s[i + 1].ord]    // noip07p2.cp
let d = eq[at].ord - '0'.ord                                            // noi03p1.cp
for (at, d) in changes: out[at] = Char('0'.ord + d)                     // noi03p1.cp
```

```
// wanted
if p1 == 2: return c.upper
var fill = [shown(c) for c in s[i - 1].next..s[i + 1]]
let d = eq[at].digit!
for (at, d) in changes: out[at] = Char.digit(d)
```

Fix, stdlib: `c.upper`, `c.lower`, `c.digit -> ?Int`, `Char.digit(d)`, and
`Char` ranges that iterate (a `Char` is already a bounded index, so
`'a'..'e'` and `.next` should come from that mechanism, not from a special
case).

## M6. Scanning text with a cursor (2 of 10: noi03p1, noip07p2)

noi03p1 is a small grammar: an optional `=`, an optional sign, a run of
digits. The port walks an index by hand, and every step has to re-check the
end of the text:

```
// today (noi03p1.cp)
var (side, i) = (1, 0)
while i < eq.len:
    var sign = side
    if eq[i] == '=':
        (side, sign, i) = (-1, -1, i + 1)
    if eq[i] == '+' or eq[i] == '-':
        if eq[i] == '-': sign = -sign
        i += 1
    let start = i
    while i < eq.len and eq[i].is_digit: i += 1
    ...
```

```
// wanted
var sc = Scanner(eq)
while not sc.done:
    if sc.eat('='): side = -1
    let sign = if sc.eat('-'): -side else: (sc.eat('+'); side)
    let run = sc.take_while(c => c.is_digit)        // a range of positions in eq
    ...
```

Fix, stdlib: a `Scanner` over a `Str` with `peek -> ?Char`, `eat(c) ->
Bool`, `take_while(pred) -> Range`, `done`. It returns positions, not
copies, because this problem needs to write back into the text at the
place a digit was found. It is forty lines of plain CP Lang; nothing about
it is special to the compiler.

## M7. Expressions cannot continue on the next line (2 of 10: ccc19s3, noi03p1)

A line cannot break after `=` or a binary operator ("expected an
expression"). ccc19s3 wanted one constant and has two:

```
// today (ccc19s3.cp)
const ROWS = [[(i, j) for j in 0..3] for i in 0..3]
const LINES = ROWS + [[(i, j) for i in 0..3] for j in 0..3]
```

and noi03p1 names two halves of a condition only to fit the line. Fix,
core grammar: a line that ends in a binary operator, `=`, or inside an
open bracket continues. This is the rule every contestant already
expects.

## M8. Three cases that differ only in which slot is empty (1 of 10: ccc19s3)

Filling the one unknown of a three-term progression is three symmetric
cases. With `?Int` cells the port unwraps all three with a dummy default
and then branches on which one was missing:

```
// today (ccc19s3.cp)
let (a, b, c) = (v[0] ?? 0, v[1] ?? 0, v[2] ?? 0)
var x = 2 * b - a
var (i, j) = line[2]
if v[0] == none:
    x = 2 * b - c
    (i, j) = line[0]
elif v[1] == none:
    if (a + c) % 2 != 0: return false
    ...
```

```
// wanted
let (k, x) = match v:
    [none, b, c] => (0, 2 * b - c)
    [a, none, c] => (1, half(a + c) else: return false)
    [a, b, none] => (2, 2 * b - a)
    _ => continue
sq[line[k]] = x
```

Fix: none proposed yet. List patterns with `none` in them would do it, and
`match` is already in the design, but this was seen once and the dummy
defaults are ugly rather than wrong. Logged so a second sighting counts.

## Seen again from batch 12

ccc18s3 (RoboThieves) was ported for this batch and dropped when batch 12
landed with the same problem. Its friction was batch 12's: a cell that
can be stepped and used as an index (L1, L5) and the missing queue (L4).
Two things from the dropped port are worth adding there. A positional
tuple does not become a named one: with `type Cell = (r: Int, c: Int)`,
passing `(0, 1)` fails with "(0, 1) has no field `r`", which pushed the
port to a struct. And the 0-1 BFS, the usual reason to want both ends of
a queue, went away once a conveyor was treated as a forced ride followed
inline, leaving a plain level-by-level BFS over two lists.

## Q1. `assume`: say the promise once (5 of 10: ioi07p2, ioi00p4, ccc19s3, coci21c6p5, ioi18p1)

`where` relates two values of one `read`. The promises in this batch are
about the whole input: walls do not cross, the regions tile the plane, the
square can be completed, the pairs split the positions. Today there is
nowhere to write them, so the port's header comment says it and nothing
checks it.

```
// today (ioi00p4.cp): the promise is prose
// each region is listed by the towns on its border, in order round it.
let border: [Side]Town = read()
```

```
// wanted (ioi07p2.cp)
let walls: [W](u: P, v: P) = read()
    assume all(pts[w.u].x == pts[w.v].x or pts[w.u].y == pts[w.v].y for w in walls)
    assume no_two_cross(pts, walls)
```

How it should work. `assume e` is a statement, legal after any `read`, and
`e` is any `Bool` expression: there is no sublanguage.

- In a debug run on real input it is an assertion. A sample you mistyped
  fails at the line that states the rule, not three phases later.
- In a judge build it is dropped.
- Under `cpl stress` with derived generators it is a filter: a case that
  fails an `assume` is redrawn, not reported. Shrinking only keeps a
  smaller case if it still passes every `assume`, so a shrunk
  counterexample is always a legal input.
- The tool counts. If fewer than about one draw in ten survives it says
  so, names the `assume` that rejects most, and suggests a `gen` (Q2).
  `assume no_two_cross(...)` over random walls rejects nearly everything;
  the contestant should learn that in the first second, not from a stress
  run that never finds a case.

This is the same split `where` already has (checked, and used to
generate), widened from a pair of values to anything. It needs no new
type-system power, which is the point: the promise stays a predicate.

## Q2. `gen`: build the input, and hold the builder to the promise (3 of 10: ioi07p2, ioi00p4, ccc19s3)

When rejection is hopeless the generator has to build a valid input.
`read(gen: f)` exists in the design for this. What is missing is the link
to Q1: with both written, the `assume` stops being a filter and becomes
the test of the generator.

```
// today: the generator lives outside the language (check.py)
def gen_ccc19s3():
    a, dr, dc, dd = R(-20, 20), R(-6, 6), R(-6, 6), R(-3, 3)
    sq = [[a + i * dr + j * dc + i * j * dd for j in range(3)] for i in range(3)]
    for i, j in random.sample(cells, R(0, 9)): sq[i][j] = "X"
```

```
// wanted (ccc19s3.cp)
fn gen_square() -> [3, 3]?Int:
    let (a, dr, dc, dd) = (rand(-20, 20), rand(-6, 6), rand(-6, 6), rand(-3, 3))
    var sq = [3, 3] (i, j) => some(a + i * dr + j * dc + i * j * dd)
    for p in sample(sq.cells, rand(0, 9)): sq[p] = none
    sq

let given: [3, 3]?Int = read(none: "X", gen: gen_square)
    assume completable(given)               // may be slow: it only runs under stress and debug
```

How it should work.

- With a `gen:`, stress calls it and then evaluates every `assume`. A
  failure is reported as "the generator broke the promise", with the case,
  and is a bug in the generator, not in the solution. This is what makes a
  hand-written constructive generator safe to trust.
- The `assume` may be expensive or even be the brute-force solution. It
  never runs on the judge.
- `gen:` fns take no arguments and read the values already in scope (`n`,
  `pts`), so a generator for the third line of input can depend on the
  first two. ioi07p2's `gen_walls` joins lattice neighbours among the
  points already generated.
- Shrinking a constructed case goes through the generator, by replaying
  it with smaller random choices, not by editing the output. Editing a
  planar map or a valid bracket word almost always breaks it.

The dropped ccc18s3 port (see "Seen again from batch 12") showed the
cheap end of the same thing. "The border is all wall and there is exactly
one S" is two lines in a generator (fill the border, place S last) and an
`assume` nobody would bother to filter on.

## Q3. Stdlib generators for the usual untypeable shapes (4 of 10: ioi00p4, ioi07p2, coci21c6p5, ioi13p4)

Four ports needed a shape that every contestant has built by hand at some
point. `check.py` has a version of each. They belong in a `std.gen` module
as ordinary fns, each returning a value that satisfies a stated predicate,
and each paired with that predicate so Q2's check applies to the library
too:

```
// wanted
use std.gen.{balanced, perm, planar_faces, disjoint_segments}

let s: Str = read(gen: () => balanced(2 * n, "()"))                 // a valid bracket word
let wiring: Perm<Switch, Door> = read()                             // derived: a bijection
let regions: List<List<Town>> = read(gen: () => planar_faces(n))    // faces of a planar map, outer face included
let walls: [W](a: Pt, b: Pt) = read(gen: () => disjoint_segments(m, box: 0..=1e6))
```

- `balanced(len, kinds)`: a uniformly random balanced word, plus
  `balanced_tree(n)` for the bracket encoding of a random tree.
- `Perm<A, B>`: a bijection with a derived generator; ioi13p4's judge uses
  it so that a wiring needs no rejection.
- `planar_faces(n)`: start from a lattice or a triangulation, delete
  random walls (which merges two faces and keeps the map planar), and
  return the faces in order round each, with the outer face. `check.py`
  does the lattice step for ioi00p4.
- `disjoint_segments(m, box)`: segments that meet only at shared ends,
  built on a lattice the way ioi07p2's sketch does.

Each is twenty to forty lines of CP Lang. None needs compiler support.
The list is short on purpose: these four are what this batch met, and a
generator should enter the library when a second port wants it.

## Q4. Generate from a witness (3 of 10: ccc19s3, coci21c6p5, noi03p1)

"A solution is guaranteed to exist" and "print any solution, or -1" both
want the same generator: build the answer first, then hide it. The witness
is worth keeping, because it turns a class of wrong answers into
failures that a checker alone cannot see.

```
// today (check.py, coci21c6p5): build a balanced choice, add decoys, forget the choice
kept = sorted(random.choice(p) for p in pairs)
... write a balanced word onto `kept`, random brackets elsewhere ...
```

```
// wanted (coci21c6p5.cp)
fn gen_case() -> (s: Str, pairs: List<(Pos, Pos)>, witness: List<Bool>):
    let pairs = random_pairing(Pos)
    let witness = [rand_bool() for _ in pairs]
    var s = Str(rand_of("()") for _ in Pos)                 // decoys
    s.write_at(chosen(pairs, witness), balanced(pairs.len, "()"))
    (s, pairs, witness)

let (s, pairs) = read(gen: gen_case)

@checker(output: ?List<Bool>)
fn ok(pick: List<Bool>) -> Bool = is_balanced(s.at(chosen(pairs, pick)))
```

How it should work.

- A `gen:` fn may return a `witness` field beside the input. It is not
  printed to the solution. Stress first runs the `@checker` on the
  witness itself, which tests the generator and the checker against each
  other before any solution runs.
- When a witness exists and the solution prints "no answer" (`-1`,
  `No`, `none`), that is a failure. Without the witness the checker has to
  accept `-1` on faith, and a greedy that gives up too early passes. In
  this batch's own check, coci21c6p5 and noi03p1 are compared with the
  original for exactly that reason.
- The reverse case is cheap to add: `gen.near(valid, moves: 1)` applies
  one random legal edit to a valid instance. noi03p1's interesting inputs
  are true equations with one match moved, which a random equation
  essentially never is; `check.py` builds them that way.

## Q5. Hidden state with a rule, in an interactor (2 of 10: ioi18p1, ioi13p4)

`@derive(Gen)` on an interactor generates its hidden state field by field.
Both interactive problems here have hidden state with a rule across
fields or positions: a word whose first letter never returns, a wiring
that is a bijection. And both have call rules that depend on `n`.

```
// today's design (plan/16-io.md shape): no place for the rule
@interactor(Combo)
@derive(Gen)
struct Judge(word: Str)                     // generates "AAAA": not a legal hidden word
```

```
// wanted (ioi18p1.cp)
protocol Combo:
    fn press(p: Str) -> Int
    limit press: n + 2
    require p.len <= 4 * n

@interactor(Combo)
struct Judge(word: Str):
    fn gen(n: Int) -> Judge: ...            // first letter, then the other three only
    fn valid(self) -> Bool = all(c != word[0] for c in word[1..])
    fn press(mut self, p: Str) -> Int = max(k for k in 0..=word.len if p.contains(word[..k]))
    fn answer_ok(self, got: Str) -> Bool = got == word
```

How it should work.

- An interactor may define `fn gen` instead of deriving it, and `fn
  valid`. They are Q2 and Q1 for hidden state: stress checks `valid` on
  every generated and every shrunk judge, and reports a generator that
  breaks it.
- `limit` takes an expression over the problem's parameters, not only a
  literal. ioi18p1's whole difficulty is `n + 2`.
- `require` on a protocol fn states what the solution must not send. A
  violation fails the run with the call log, the same as an exceeded
  limit. Without it a solution that presses 5n buttons passes locally
  and fails on the judge.
- Where a stdlib type already carries the rule, deriving is enough:
  ioi13p4's `door: Perm<Switch, Door>` needs no `gen` and no `valid`.
- An adaptive judge keeps "the set of states still consistent"; with
  `valid`, that set is filtered by the rule for free.

## Q6. Ranges that are legal but never collide (2 of 10: ccc00s5, ioi00p4)

The bugs in ccc00s5 are all ties: two sheep at the same distance from a
point of the fence, a sheep straight above another. With coordinates drawn
from the statement's `0..=1000` with two decimals, a tie essentially never
happens, and stress passes a solution that uses `<` where it needs `<=`.
`check.py` draws from a 13 x 12 lattice instead. That is neither an
`assume` (wide inputs are legal) nor a constructive generator (nothing is
being built).

```
// today (check.py)
sheep.add((R(0, 12) * 25, R(1, 12) * 25))
```

```
// wanted
let sheep: List<(x: Fixed<2>, y: Fixed<2>)> = read(n)
    stress x in 0..=3 by 0.25, y in 0.25..=3 by 0.25
```

Fix: a `stress` clause that narrows what the derived generator draws
without changing what the program accepts. Edge-biased generation already
favours the ends of a range; this is the missing way to say "small and
coarse", which is what makes equal values, collinear points and shared
walls appear. One clause, on the `read`, next to `where` and `assume`.
The three then read as a set: `where` and `assume` say what is legal,
`gen:` says how to build it, `stress` says where to look.
