# Friction Backlog — Scout Batch 40 (Canadian and DMOJ classics)

Ports are in `proto/ports/scout_batch_40/`; the v2 reference is `docs/one-page-cheat-sheet.md`. This batch covers the `ecoo*`, `dwite*`, `vm7wc*`, `tle*`, `gfssoc*`, `mwc*`, `valentines*` and `cheerio*` prefix bucket.

## The bucket is nearly empty

The request was ten hard problems. The corpus has ten codes in this bucket in total, and only five were portable: `cheerio1s1`, `cheerio1s2`, `cheerio1s5`, `tle17c5p1` and `tle18p5`. Four others (`cheerio1s3`, `cheerio1s4`, `dwite10c3p3`, `tle17c5p6`) were already ported, `dwite08c5p4` exists only as JSFuck that prints five constants, and no corpus code starts with `ecoo`, `vm7wc`, `gfssoc`, `mwc` or `valentines`. With these five the bucket is exhausted; a future scout should not be pointed at it again.

They are also not hard. Three are Cheerio S1/S2/S5 and one is a one-question interactive. Only `cheerio1s5` (two stacked DPs) and `tle18p5` (a grader with a compile-time constant) put any pressure on the language, so the findings below are few and should be weighed as such.

## Dual LOC

Output of `python3 proto/ports/scout_batch_40/loc.py` (non-blank, non-comment lines; "stripped" removes the contest template and stream setup from the original):

```
problem      raw stripped   v2 today
cheerio1s1    49       21   10    12
cheerio1s2    51       23    9    11
cheerio1s5    74       46   16    20
tle17c5p1     36        9   10     2
tle18p5       82       82   18    18
total        292      181   63    63
v2 vs raw: -78%   v2 vs stripped: -65%
today vs raw: -78%   today vs stripped: -65%
```

Two rows need reading with care. `tle17c5p1` in v2 is 2 lines of solution plus an 8-line `@interactor` judge that the original does not have; without the judge the v2 total is 55 lines (-81% raw, -70% stripped). `tle18p5` has no template to strip, and its 82 lines are a template-metaprogramming stunt forced by the problem (no headers, constant `N`), so its -78% says more about the original than about CP Lang.

## What read cleanly

`cheerio1s1` is the kind of problem v2 is already right for. A record type gives the dependent bound on the position and the loop destructures it:

```cp
let (n, k): (1..=2e5, 1..=1e9) = read()
type Light = (p: 1..=k, r: 1..=1e9, g: 1..=1e9)
let lights: [0..n]Light = read()

var (pos, t) = (0, 0)
for (p, r, g) in lights:
    t += p - pos
    pos = p
    let phase = t % (r + g)
    if phase < r: t += r - phase
print(t + k - pos)
```

`cheerio1s2` got shorter than a line-by-line translation because the port was rethought. The original keeps `pair<long, int>` in the heap plus a `rows[]` counter to compute the row's next price. The next seat in the same row always costs `b` more, so the heap needs only the price:

```cp
var next = MinHeap<Int>()
for i in 1..=n: next.push(a * i + b)
var total = 0
for _ in 0..n:
    let w = next.pop()!
    total += w
    next.push(w + b)
print(total)
```

`cheerio1s5`'s interval DP is one expression under `@memo`, and half-open `l..r` removes both `if (i > l)` / `if (i < r)` guards the original needs for its inclusive indices:

```cp
@memo
fn clear(l: 0..=n, r: 0..=n) -> Int:
    max(r - l + clear(l, i) + clear(i + 1, r) for i in l..r if s[i] == 'H') ?? 0
```

`proto/cpc` already builds this shape (`@memo` plus `max(gen) ?? 0`); the `today/` file uses bottom-up tables only because of C2 below.

## Friction

### C1. A judge-supplied compile-time constant has no `@grader` form (`tle18p5`)

The judge's header defines `N`, `get_item` and `print_num`. The cheat sheet gives `@grader fn` (bodyless = extern) and `@grader struct`, but nothing for a constant the grader owns. The port wants the array domain to be that constant:

```cp
@grader const N: 0..=2000                 // WISH: not in the cheat sheet
@grader fn get_item(i: Int) -> Int
@grader fn print_num(x: Int)

domain I = 0..N
let a: [I]Int = [I] i => get_item(i.to_idx)
```

This is the same collision batch 37 logged for grader parameters that size a domain, in a new position: there the size arrived as a function argument, here as an extern constant. Options: allow `@grader const NAME: T` (and `@grader let` for a non-constant extern global), or say in the cheat sheet that such problems are out of scope. One line in section 8 would settle it. A second, smaller point: `[I] i => get_item(i.to_idx)` is the only place `.to_idx` appears, and it is there solely because the extern takes a raw `Int`.

### C2. `Str` of unstated length against `Int` positions (`cheerio1s5`)

The input is a string with no length line, and the DP works on the `n + 1` gaps between characters with plain arithmetic (`r - l`, `i + 1`, `heads.sum(i..j)`). The v2 port writes

```cp
let s: Str = read()
let n = s.len
...
for i in l..r if s[i] == 'H'
```

and the cheat sheet does not say whether that is legal. If `Str` is `[domain S]Char`, then `s[i]` with `i: Int` should be `s.raw[i]`, and the port either sprinkles `.raw` or converts every gap to an `S` index. Today's prototype rejects both direct spellings:

```
let s: Str = read();  s[0]            -> error: cannot index Str
let s: [domain S]Char = read();  s.raw[i]   -> error: [S]Char has no `.raw`
```

so `today/cheerio1s5.cp` copies the string into a `List<Bool>` first:

```cp
let s: [domain S]Char = read()
let n = S.len
var head: List<Bool> = []
for i in S: head.push(s[i] == 'H')
```

That copy is 3 of the 4 extra lines in the `today` column. Recommendation: state in the cheat sheet that a `Str` read without a domain is `Int`-indexed (`[0..s.len]Char`), so that gap-based string DPs need neither `.raw` nor a named domain. Interval DP over string gaps is common enough that this should be the zero-ceremony path.

### C3. Three features meet in the block-selection DP (`cheerio1s5`)

```cp
@memo
fn best(i: 0..=n, used: 0..=k) -> Int:
    if i == n: return 0
    var res = best(i + 1, used)
    for j in i + 1..=n:
        let now = used + heads.sum(i..j)
        if now <= k: res max= clear(i, j) + best(j, now)
    res
```

`best(j, now)` passes `now: Int` where the parameter is `0..=k`, guarded by `if now <= k` on the same line. The cheat sheet says ranges on scalars are checked bounds, but not whether a `@memo` parameter range narrows from a preceding comparison or traps at the call. Either is workable; the user needs to know which, because the natural mistake is to drop the guard and expect `?? -inf` behaviour. Also unspecified: `i + 1..=n` parses as `(i + 1)..=n` here, which is what every port assumes; the precedence of `..` against `+` deserves one explicit line.

### C4. `max_by` on an empty domain and on ties (`tle18p5`)

```cp
var at = max_by(I, i => len[i])
while let i = at:
    path.push(a[i])
    at = prev[i]
```

This is correct only if `max_by` returns `?I` (so `N = 0` prints just `0`) and it matches the original's output only if ties go to the first maximum. The cheat sheet says `min(gen)` / `max(gen)` return `?T` but lists `min_by` / `max_by` without a return type or a tie rule. Proposal: `min_by` / `max_by` return `?T`, first extremum wins, written next to the signature. `proto/cpc` has no `max_by` yet (`unknown name max_by`), so the `today/` file tracks the best end by hand.

### C5. Prototype gaps seen on the way (not v2 issues)

These are differences between the cheat sheet and `proto/cpc` today, listed so the next scout does not rediscover them:

```
let (n, k): (1..=2e5, 1..=1e9) = read()    -> error: unknown name read
for j in ..n if a[j] > 0: ...              -> error: expected ':' (at 'if')
```

Tuple-typed `read()` and the filtered `for ... if` loop both cost a line each in the `today/` files. Record destructuring in `for (p, r, g) in lights`, `@memo` on range-typed parameters, `max(gen) ?? 0`, `MinHeap<Int>` and `ask(...)` all build as specified.

## No friction found

`tle17c5p1` is two lines in both forms, and `@interactor` plus `read_output()` expressed the judge directly. `cheerio1s1` and `cheerio1s2` needed nothing beyond the basics. Nothing in this batch argues for a new construct; C1, C2 and C4 are each a one-line clarification in the cheat sheet, and C3 is a question about existing semantics.
