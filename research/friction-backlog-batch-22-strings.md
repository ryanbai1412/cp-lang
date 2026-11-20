# Friction backlog, batch 22: hard strings, automata and parsing

A scout pass over ten submissions where the hard part is string-shaped:
a suffix automaton (noi18p3), a trie (ioi08p1), searches whose states are
strings (cco19p6, ccc01s5), string DPs (ioi16p4, noi09p2), letter-pair
counting (ioi21p4) and three kinds of parsing (ccc99s5, noi14p1,
noi21p4). Two more ports, ceoi22p2 (a `min`/`max` expression) and
noimps22p1 (a C preprocessor), were written, passed the same check, and
were then dropped because batch 19 landed with both; what they showed
is kept below and marked "dropped port". The ports are in
[`proto/ports/scout_batch_22/`](../proto/ports/scout_batch_22/). Items are
numbered `X1`... so they do not collide with earlier batches. Batch 10
(`T1`...) covered KMP, Z, suffix arrays and hashing; where an item here
repeats one of those, it names it and adds only what is new.

What was checked: every port agrees with its original on 60 small random
inputs (`python3 proto/ports/scout_batch_22/check.py`, on the reference
interpreter at commit `9df888a`). Every "today" snippet is a line a port
contains, and every error quoted came from a probe on that interpreter.
"Wanted" snippets do not run. Nothing was run at full input size, and
nothing was run on the Rust compiler or `proto/cpc/`. The README of
the batch says how far the work was statement-first (only partly); the DMOJ API answered
403 for every problem.

## The headline: `Str` is open in the interpreter, and thin

Blind critic 1 found `Str` opaque on `proto/cpc/`: thirteen probes
failed, starting with `s[0]`. On the reference interpreter the rule the
synthesis asks for ("a `Str` opens wherever it is stored") already holds.
All of these ran, on a `Str` in a `let`, an array element, a list element,
a map key, a record field and a function argument:

```cp
let words: [Word]Str = read()                    // ioi08p1, cco19p6
let d: [D](c: Char, x: Str) = read()             // the critic's dictionary; d[i].x[0] works
var dist = Map<Str, Int>()                       // cco19p6: a string as a vertex
var queue = Heap<(Int, Str)>(Order<(Int, Str)>.by(p => p.0))
for c in word: ...                               // every port
s[i], s[a..b], s.len, s.rev, s.last, s + t, s == t, s < t, t.push(c), t.extend(u)
sorted(s), min(s), Set(s).len, max(c for c in s), s.contains(c)
```

So no port in this batch fought the type. What they fought was everything
one layer up: there is no line reader, no open-ended slice, and not one
string helper (`starts_with`, `find`, `split`, `join`, `count`, `upper`).
The ports wrote by hand several helpers a contestant would expect to
exist (X3, X10). The gate for the `Str` work should therefore be on the
compiler side (make cpc and Rust do what cpi does), and the items below
are the library and syntax that a string problem then needs.

## Suggested order

1. X1 `read_line()`: without it a whole problem class cannot be run.
2. X2 open-ended slices, X3 the prefix and search helpers.
3. X4 a copy on `var t = s` (a wrong-answer trap, not an inconvenience).
4. X5 `Trie<A>` and X6 `SuffixAutomaton<A>` in `std.strings`.
5. X9 `U64` in the interpreter; X7, X8 and the smaller ones after.

## X1. There is no way to read a line (dropped port noimps22p1; batch 19; `cco04p2`)

`plan/16-io.md` has `read_line()`. The interpreter splits all input on
white space before the program starts, so a line with spaces cannot be
recovered. *Checked:* `let line: Str = read_line()` is "unknown name
`read_line`". The preprocessor port is written as designed:

```cp
let n = Int(read_line())
for _ in 0..n:
    let line = read_line()
    if not line.is_empty and line[0] == '#':
        let (directive, rest) = cut(line, ' ')
```

and was checked through a copy with a token-reading stand-in, fed the
input with spaces written as `~`. That tests the logic and nothing about
the reader. Batch 19 reports the same gap on `proto/cpc/`. Three things the real one must settle, which the port had to
guess: (a) after `let n: Int = read()`, does `read_line()` return the rest
of the first line (empty) or the next line? C++ contestants lose minutes
to exactly this with `cin >> n; getline`. Proposal: `read_line()` after a
token read skips to the next line if only white space is left on this one.
(b) an empty line is `""`, not end of input. (c) a trailing `\r` is
dropped.

## X2. Slices must be closed at both ends (cco19p6, ccc01s5)

Memory of every earlier batch says `s[..i]` and `s[i..]` are wanted. On a
`Str` neither works, and one of them crashes the interpreter instead of
reporting:

```cp
print(s[2..])     // error: cannot build a range from 2 to none
print(s[..2])     // Python traceback: 'NoneType' object has no attribute 'kind'
```

Today:

```cp
if starts_with(a, b): some((a[b.len..a.len], b.len))        // cco19p6
if s[i] == sep: return (s[0..i], s[i + 1..s.len])           // dropped port noimps22p1
```

Wanted: `a[b.len..]`, `s[..i]`, `s[i + 1..]`. Naming the string twice is
where the wrong variable gets in: ccc01s5 has
`long[short.len..long.len]` and `short[long.len..short.len]` two lines
apart.

## X3. No prefix test, no search, no split, no join (4 of 10)

*Checked,* each is "has no method": `s.starts_with("he")`, `s.find("l")`,
`s.split("l")`, `"-".join([s, s])`, `s.count('l')`, `s * 2`. What the
ports wrote instead:

```cp
fn starts_with(s: Str, head: Str) -> Bool = head.len <= s.len and s[0..head.len] == head   // cco19p6, ccc01s5
fn cut(s: Str, sep: Char) -> (Str, Str): ...                // dropped port noimps22p1, 4 lines
fn runs(line: Str) -> List<Str>: ...                        // the same, 6 lines
print(Str([Char(Int('0') + digit[Letter(c)]!) for c in row]))   // ccc99s5: no join
var text = ""                                               // noi09p2: no join, 4 lines
for i in breaks[k] + 1..=breaks[k + 1]:
    if not text.is_empty: text.push(' ')
    text.extend(words[Sentence(i)])
```

`starts_with` was written twice with the same body, and its guard
`head.len <= s.len` is needed only because a slice past the end stops
the program (*checked:* "slice 0..9 is out of bounds for length 5"). Wanted, all userland over `List<T>` so they cost no string
rule: `starts_with`, `ends_with`, `find(pat) -> ?Int`,
`split_once(sep) -> ?(Str, Str)`, `split(sep)`, `join`, `count`, and
`group_by(key)` for noimps22p1's `runs` (runs of equal `in_name(c)`). The
two overhang ports want one more, which is the actual idea of both
problems:

```cp
/// Put b beside a: what sticks out and how much is shared, or none if they disagree.
fn overhang(a: Str, b: Str) -> ?(Str, Int)                  // cco19p6
```

It is `10-T5` ("how much of a pattern ends here") seen from the other
side; a `common_prefix(a, b) -> Int` in `std.strings` would make it one
line and also serves ccc01s5.

## X4. `var t = s` does not copy (probe)

*Checked:*

```cp
let s: Str = read()      // hello
var t = s
t[0] = 'J'
print(s, t)              // Jello Jello
```

`s` is a `let` and it changed. *Checked:* the same holds for a list
(`let s = [1, 2]`, `var t = s`, `t[0] = 9` prints `[9, 2] [9, 2]`). No
port in the batch depends on it: ccc01s5 passes the overhang as an
argument and builds new strings with `+`, and ioi21p4 writes
`upto.last.copy()`. But a backtracking search that saves a string,
extends it and restores it on failure would restore nothing, with no
message. With no stored borrows in the design, a binding should be a
copy; this looks like an interpreter bug and deserves a test in
`proto/cpi/checks_test.py`.

## X5. A trie, again (ioi08p1; `10-T10`, `7-D4`)

Third batch to hand-build one. The eight lines are the same as
sac22cc3p5's to the letter:

```cp
var at = 0
for c in word:
    if let next = trie[at].kid[Letter(c)]: at = next
    else:
        trie.push(Node([Letter] c => none, false, false))
        trie[at].kid[Letter(c)] = some(trie.len - 1)
        at = trie.len - 1
trie[at].word = true
```

New in this batch: the node needs data the library cannot know (`last`,
"on the way to the word printed last"), and the walk is a DFS in letter
order with one child deferred. So `Trie<A>` has to be `Trie<A, T>` with a
payload, give node ids that index user arrays (`[trie.Node]Bool` would do
instead of a payload), and expose `kids(node) yields (A, Node)`:

```cp
var trie = Trie<Letter>()
for w in words: is_word[trie.insert(w)] = true
for (c, next) in trie.kids(at): ...
```

`trie.insert(w)` returning the end node, and `trie.walk(w) yields Node`
for marking a path, cover both loops of ioi08p1. Also seen: `c.label`
turns a `Letter` back into a `Char`. It works; nothing suggests the name.
`Char(c)` would be the guess.

## X6. A suffix automaton (noi18p3)

No earlier batch had one. The port's `Automaton` is 30 lines and uses only
structs, lists and `[Letter]?Int`, so it belongs in `std.strings` as
written. What using it showed:

```cp
struct State:
    len: Int
    link: Int           // the root links to itself
    next: [Letter]?Int
    first: Int
```

- The link wants to be `?State`, but then every `states[states[v].link]`
  needs a `!`. The port links the root to itself and breaks the two
  walks on `p == 0`. A library version should expose
  `up(v) yields State` (v, its link, ... , the root) so users never
  write that loop.
- States are numbered by a growing list, so they are plain `Int`s and the
  per-state table of the user (`ends`, the segment tree roots) is a
  `List<Int>` kept in step by hand:
  `while ends.len < of_s.states.len: ends.push(0)`. With the sealed
  runtime-frozen index (`let K = n`) this is `[sa.State]Int` once the
  automaton is built. The automaton is built incrementally while the
  table is filled, which the frozen index cannot express; a
  `sa.freeze()` step, or building first and filling after, is needed.
- "Longest states first" is `longest_first.sort(by: v => -of_s.states[v].len)`.
  Every suffix-automaton solution needs this order; ship it as
  `sa.bottom_up`.
- The persistent merge tree is three lists and three recursive
  functions. A first draft had
  `left.push(union(left[a], left[b]))` followed by the same for `right`:
  the inner calls push to both lists between the two outer pushes, the
  lists fall out of step and the tree gets a cycle. The interpreter died
  with a Python `RecursionError` rather than its own "recursion deeper
  than 200 calls" message. One list of `(left, right)` records would
  have made the bug impossible; `Pool<Node>` (asked for in `10-T10`)
  is the fix, and the sparse mergeable set is common enough
  (`std.segtree` has no dynamic or mergeable form) to be a library type:
  `SparseSet.single(i)`, `a | b` persistent, `s.any(range)`.
- The match loop reads well and is the part worth protecting:

```cp
if let next = of_s.states[at].next[c]:
    if any_in(ends[next], 0, n, from + len, to):
        (at, len) = (next, len + 1)
        break
```

## X7. No `any` / `all` (ioi21p4, noi21p4, ioi16p4)

*Checked:* `any(c == 'l' for c in s)` is "unknown name `any`", while
`sum(... for ...)` and `max(... for ...)` work. Three ports carry a flag:

```cp
var possible = true
for s in Base:
    if sum(have[s, t] for t in Base) != sum(have[t, s] for t in Base): possible = false
```

Wanted: `if any(... for s in Base):`. noi21p4's flag also hides that the
loop should stop at the first hit.

## X8. Fixed small arrays and counted reads (ioi08p1, ccc99s5)

*Checked:* `let words: [n]Str = read()` and `let rows: [0..3]Str = read()`
are "`3` is not an index type"; `read(3)` for a `List<Str>` is "unknown
name `read`" with an argument. So three words need a named domain, and
ccc99s5, which steps through rows by number, copies the array out again:

```cp
domain Row = 0..3
let read_rows: [Row]Str = read()
let rows = [read_rows[r] for r in Row]
```

Wanted: `let rows: List<Str> = read(3)` (already item 6 of the synthesis).
A domain for "the three words of a sum" names nothing.

## X9. No `U64`, so a C generator cannot be typed (noi21p4)

The statement gives its generator as C on `unsigned long long`:
`k3 ^= k3 << 23; k2 = k3 ^ k4 ^ (k3 >> 17) ^ (k4 >> 26); return k2 + k4;`.
`plan/11-values-and-types.md` has `U64` that wraps. The interpreter has
only `Int`, which traps (*checked:* "Int overflow: 9223372036854775807 + 1
does not fit in 64 bits"), and has no `~` (*checked:* "expected an
expression (at '~')"). The bare-core fallback is `bits.cp`: a `U64` of
two 32-bit halves with `xor`, `shl`, `shr`, `plus` and `bit` as methods,
because a userland struct cannot take `^` or `<<`:

```cp
k3 = k3.xor(k3.shl(23))
k2 = k3.xor(k4).xor(k3.shr(17)).xor(k4.shr(26))
k2.plus(k4).bit(32)
```

Two further holes around it. The seeds go up to 2^64 - 1 and are read as
`Str` and parsed by hand; *checked:* `let a: Int = read()` on
`18446744073709551615` prints it back, and `Int("12345678901234567890")`
likewise, so the interpreter's `Int` silently holds values the compiled
program cannot. Both should stop with the input position. And the
question's 256-bit word is `[Block]Int` with sixteen 16-bit blocks; with
`U64` it would be four words, or `Bitset<256>` with `^` and `count`
(`8-H4`), which is the type the problem is about.

## X10. Hex digits and character classes (noi21p4, ccc99s5)

`Char` has `is_digit`, `is_lower`, `is_upper` and `c - 'a'`. Missing,
each *checked* as "has no field or method": `is_alpha`, `upper`, `lower`,
and any digit value. Written by hand:

```cp
fn hex(c: Char) -> Int = if c.is_digit: c - '0' else: 10 + (c - 'A')             // noi21p4
fn in_name(c: Char) -> Bool = c.is_lower or c.is_upper or c.is_digit or c == '_' // dropped port noimps22p1
```

Wanted: `c.digit(16) -> ?Int` (and `Int.parse(s, base: 16)`),
`c.is_alpha`, `c.is_alnum`. The synthesis lists `c.digit`; the base is
what is new. Going the other way, ccc99s5 builds a digit character as
`Char(Int('0') + d)`; `Char.digit(d)` is in the synthesis and is needed.

## X11. Named tuples lose their names in a list (dropped port ceoi22p2)

```cp
type Room = (below: Int, above: Int)
var vals: List<Room> = []
vals.push((1, 1))
vals[0].below          // error: (1, 1) has no field `below`
```

The literal is not given the type the list expects. The port uses
`struct Room(below: Int, above: Int)` and `Room(1, 1)` instead, which is
fine, but the error points at the read, three lines from the cause.

## X12. Small syntax holes hit while writing

Each *checked* on the interpreter; each cost one rewrite.

- A function whose `=` body continues on the next line:
  "expected an expression (at None)". ccc99s5's `at` became a block.
- `for (b, i) in Block * 0..4:` is "no `*` for Block and 0"; a nested
  loop instead (noi21p4).
- `m[key].push(x)` on a missing key is "key a is not in the map", though
  `m[key] += 1` on a missing key works. noi21p4 guards with
  `if not sharing[b].contains(word[b]): sharing[b][word[b]] = []`. Either
  both default or neither; `m.entry(key)` or a `Map` built with a default
  would settle it (`1-F18`, `10-T11`).
- The dropped noimps22p1 wrote `out[out.len - 1].push(c)` to extend the last
  string of a list, out of doubt whether `out.last` is a copy.
  *Checked* afterwards: `out.last.push(c)` does change the list. Given
  X4 that is the same sharing, so the rule for `last` needs stating.
- `print()` prints an empty line, and `print(op)` for a `Char` prints it
  bare: both as hoped. `print([..chars..])` prints `[A, B]`, so a list of
  characters must be wrapped in `Str(...)` to print as text.

## X13. Parsing without a scanner was fine

noi14p1 (`(Str, Int)` tokens), ccc99s5 (letters by column), noi21p4 (hex
digits) and the dropped ceoi22p2 (two stacks keyed on single characters)
needed no parsing support at all:
typed `read()` plus `for c in s` was the natural first draft and it was
right. Only the dropped noimps22p1 wanted tokenizing, and there `group_by` (X3) would
do. So this batch gives no evidence for the forty-line `Scanner` of the
synthesis; it should wait for a port that needs positions back.

One real limit: ceoi22p2's expression is two million characters and nests
a million deep. A recursive-descent parser, the textbook answer, cannot
run in the interpreter (*checked:* a function recursing 500 deep ends in
a Python `RecursionError`, not the interpreter's own depth message) and
risks the stack on a judge. The port's stack form is shorter anyway,
but the compiler should say which functions recurse on input size.

## X14. Stress constraints that cannot be typed

- ioi16p4: "at least one painting fits the clues". The generator in
  `check.py` builds a painting first and hides cells; a random row with
  random clues is almost never valid. This is "generate from a witness"
  (`Q4`), and the witness is also a check on the answer: every cell the
  port calls `X` or `_` must agree with it.
- ccc99s5: "exactly one answer". Not typeable and not cheap to generate;
  `check.py` drops it and judges any valid assignment, which means the
  uniqueness the statement promises is never tested.
- noi21p4: the dictionary is the output of a fixed generator, and the
  solution is only fast because that output is random. A stress input
  cannot choose the dictionary, only the seeds, and the questions must be
  built from the dictionary (a word with a few bits flipped) or every
  answer is 0. `check.py` re-implements the generator in Python for this.
  The chained flip ("after a 1, the next word is inverted") means a
  generator cannot aim a question without knowing the previous answer:
  an online-encoded input needs the reference solution inside the
  generator.
- cco19p6, ioi08p1: "all words different" is `Set<Str>` in the generator;
  as a type it would be `[Word]Str where distinct`, which `S4`-style
  refinements already cover.
- noi18p3: "the lengths of all t add up to at most 1e6" is a bound on a
  sum over queries; no per-field range says it.
- noi09p2: the answer is "Too hard to arrange" above 1e18, so costs
  must be compared past the range of `Int`, which traps. The port
  compares in `Float` and adds the printed cost up again in `Int`; a
  saturating `Int` (or `min(x, cap)` that the range checker accepts
  before the multiply) would let one type do both. The generator needs
  a second mode with a huge `l` and `p` to reach that branch at all, and
  the lines printed are one of many optimal settings, so the judge
  recomputes the cost of whatever is printed.

## Already good

- Strings as map keys, heap entries and function arguments, compared and
  sliced by value, made cco19p6 thirty lines. The original interns every
  string to an integer by hand.
- `domain Letter = 'a'..='z'` with `[Letter]?Int` and `Letter(c)` reads
  a wrong character as an error at the conversion, not as a stray index.
- `if let next = trie[at].kid[c]: ... else: ...` is the right shape for
  every automaton step in the batch.
- `[Base, Base] (s, t) => upto[y + 1][s, t] - upto[x][s, t]` (ioi21p4):
  a table of nine counts as one expression.
- `s.rev` and `lens.rev` let ioi16p4 run one DP twice instead of writing
  its mirror image, which is where the original has its second copy.
- Closures as continuations (`give(x, d, () => fill(...))`) made
  ccc99s5's assign, recurse, undo a single function.

## Line counts

Non-blank, non-comment lines of the port against non-blank lines of the
original, which includes its template: noi18p3 91 / 537, ioi08p1 42 / 87,
cco19p6 30 / 97, ccc01s5 22 / 72, ioi16p4 50 / 125, ioi21p4 26 / 88,
noi09p2 42 / 244, ccc99s5 36 / 98, noi14p1 16 / 102, noi21p4 37 / 121. Library-stripped counts were not measured for this
batch. noi21p4's count leaves out `bits.cp`, which exists only because
of X9.
