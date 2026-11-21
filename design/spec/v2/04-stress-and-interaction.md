# Spec chapter 4: alternatives, checkers, generators and interaction

Status: **draft v2** [proposed]. Brutes live in the same file and are stripped from submissions [you].

## 4.1 Alternatives

```cp
@alt(solve, n: ..=8)
fn slow() -> Int: ...
```

- `@alt(f)` marks another implementation of `f`: a brute or a second fast algorithm. `cp stress` runs each
  against `f` on generated inputs.
- **Size caps** name a program-level read or domain (`A: ..=8`) or, qualified, a `gen` binding
  (`input.n: ..=10`) [r4 16]. An uncapped alt uses the `stress` context's generic defaults
  (`with stress(max_size: 1000, time: 10s)`); caps live only on `@alt` [r10 22]. When both are static, a cap
  that excludes every generated input is a compile error; otherwise stress checks at start-up [r10 Q4.1].
- **Inside the alt, a cap refines the type** of what it names (`n: 1..=8`), so `Bits<0..bets.len>` meets
  `Bits`'s static ≤ 64 rule and widths shrink [r11 N3]. Entering a capped alt converts each capped value with
  a check, trapping with "input exceeds the alt's cap `n: ..=8`"; stress only generates within caps [r12 R3].
- An inner function's alt is substituted at the real call sites in whole-program stress and compared there,
  so its preconditions hold. Stressing it alone needs its own `@gen` [r4 19, 26].
- A function that mutates captured state can't have an `@alt`, except state declared `scratch var`: a buffer
  whose contents don't survive between calls. Debug poisons it on entry from outside the function's recursive group, never on recursive calls; `@memo`
  functions can't touch `scratch`. Stress runs twice with different poison patterns, so a result that
  depends on stale contents shows up [p1 Q4, r13 P9].
- **Whole-program alts** [p1 F9]: `@alt(program, caps)` on a function returning nothing prints like the
  program; stress compares output.
- Alts may capture per-test state (they are substituted where `f` is called); other tool roots can't. An alt of
  a nested function is declared in the same block, so it sees the same captures [p3 proposed].

## 4.2 Checkers

- **One shape, a relation:** `check: (input, ours, theirs) => Bool`, passed in `@alt(…, check: …)`. Equality
  is the default.
- **Unary:** a program-level `@check fn ok(input: In, output: Reader) -> Bool`, where `In` is the `@gen`
  return type or a `Reader` over the input text (for a grader: the argument tuple and return type) [r10 25]. It runs on every implementation's
  output; the relation runs between `f` and each alt [r4 17].
- A unary checker can't catch a wrong "impossible", so problems with an impossible answer also get a brute
  `@alt` with a relation comparing feasibility [r3].
- Floats: `approx(a, b, abs:, rel:)` is an ordinary function and the only float equality [r7].

## 4.3 Generators

- One system, `gen<T>(…)`, driven by types: `gen(1..=8)`, `gen<[V]1..=1e9>()`, `gen<Graph<V>>(simple: true)`,
  `gen<Perm<1..=n>>()`.
- Constraints types can't state go in `@gen fn input()`, which writes the input with `print` exactly as the
  judge would.
- The shrinker records every `gen` choice and minimizes them, so a failing case shrinks without a
  per-problem shrinker.
- `cp stress` reports: the smallest failing input, each implementation's output, and the first differing
  line, with values in the file's base.

## 4.4 Interaction

```cp
let judge = Wire(error_reply: -1)
fn neighbour(v: V) -> V = judge.ask("? {v}")         // exits on the raw error reply, so `V`, not `?V`
judge.say("! {answer}")
```

- An `@interactor` is an ordinary program run by `cp stress` on the other end of the pipes. It sees the
  program-level context, not per-test state.
- The stdlib `Wire` owns error replies, query limits and flushing. An interactive error reply is compared on the raw
  token, before base conversion.
- **Flush rule:** a program in which `Wire` is reachable after stripping (including through libraries), and
  every `@interactor`, flushes pending output before
  every read and at exit, including after `fail`. This is decided at compile time on the program after alts
  and tool roots are stripped, and local runs use the same mode [r4 13, r5 10]. Other programs never flush
  early, so ordinary output stays buffered.
- `protocol:` and `messages` blocks are rejected. The wire format is an ordinary payload enum with one
  `Read + Print` impl, shared by both sides [p3]:

```cp
enum Query<V>: ask(V), answer([V]V)

impl Read + Print for Query<V>:
    fn print(q) = match q:
        ask(v) => "? {v}"
        answer(c) => "! {c}"
    fn read() = match read<Byte>():
        '?' => ask(read())
        '!' => answer(read())
        other => fail("bad token {other}")

var judge = Wire<Query<V>>(error_reply: -1, limit: [asks: V.count])
let u: V = judge.query(.ask(v))          // the reply type comes from the annotation, as with read()
judge.say(.answer(colour))
```

- `Wire<Q>` defaults `Q` to raw text (`Wire()` takes strings), so the typed form is opt-in; `wire.read<Q>()` reads one typed message from an untyped wire. Traces show each exchange as
  `ask(3) → 7`.
- **Limits belong to the protocol [p3].** They are declared once on the message type and enforced by both
  sides:

```cp
impl Limited for Query<V>:
    type Kind = labels(asks)                 // one budget per kind
    fn kind(q) -> ?Kind = match q:
        ask(_) => asks
        answer(_) => none                    // not counted
    // optional: fn cost(q) -> Int, default 1, for weighted limits
```

  - `Wire<Q>(limit: …)` exists only when `Q: Limited`, and takes a value for each `Kind`.
  - Solution side, debug builds: exceeding a limit traps at the calling line ("ask #1001 exceeds limit 1000").
    Release builds don't count.
  - Interactor side: the same `limit:` makes its `Wire` reply `error_reply` and fail the run, so there are no
    hand-written counters.
  - A limit shared across tests is an untyped `static` `Wire` with a typed view per test (4.6b).
- **Hidden tests are values [p3].** The interactor receives the hidden test as a parameter instead of
  generating it. A `@gen` builds it, so a failing test can be saved, replayed, shrunk and hand-written:

```cp
struct Case: n: 1..=8, edges: List<(Int, Int)>       // plain data: no fresh domain escapes the @gen
@gen fn hidden() -> List<Case> = ...
@interactor fn judge(tests: List<Case>, wire: Wire): ...   // per case: `domain V = case.n`, then `wire.typed<Query<V>>(limit: …)` and `w.read()`
```

  The interactor still prints what the real judge prints first (t, n, degrees). Its first parameter has the
  type the `@gen` returns: a `Case` for a single-test problem, a `List<Case>` for multitest (it prints `t`).
  With several `@gen`s, `@interactor(gen: hidden)` pairs them [r16 R14].
- **One process under stress [p3].** `cp stress` and `cp run --judge` run the solution and the interactor
  as two coroutines in one process, connected by an in-memory `Wire`. Runs are deterministic and produce
  one interleaved trace. Submissions use real stdin/stdout, and the flush rule applies unchanged.

## 4.5 Graders

`@grader fn init(…)` and similar export functions with the judge's C signature. Grader arguments never
convert under `io(base:)`. `@gen` for a grader problem produces the argument tuple. Stress runs the grader
functions through a generated harness.

`@judge_fn(limit: n + 2) fn press(p: Str) -> I32` declares a function the judge provides. Stress
substitutes the `@mock(press)` implementation and counts calls against the limit, trapping at the calling
line in debug, the same as `Wire(limit:)` [p1 F12, r13 P16, p3].

## 4.6 Lowering and bundling

Alts, checkers, generators and interactors are stripped from the submission. The bundler confirms by
re-parsing the emitted C++ that no stripped symbol remains. Debug and release must agree on every stress
input (goal V).

- **Debug and release agree on every input where debug doesn't trap.** Overflow and out-of-range shifts are
  the cases where they can differ [r12 R13].

## 4.6b Round-15 interaction fixes [p3]

- **One interactor signature:** `@interactor fn judge(tests: List<Case>, wire: Wire)`. Per-case typing and limits
  use a view: `let w = wire.typed<Query<V>>(limit: [asks: V.count])`. The raw stdio is one resource.
- **Shared limits across tests:** `static var raw = Wire(error_reply: -1, limit: …)` (untyped, so legal as a
  static), with `raw.typed<Query<V>>()` per test. Limits live on the raw wire.
- **Limits are a `[Kind]Int` literal:** `limit: [asks: V.count]`.
- **Handles:** `Wire` counts, so it is a `var`. The solution calls `judge.query(.ask(v))`. A leading-dot
  variant `.ask(v)` is allowed wherever the expected type is a known enum.
- **No per-message allocation:** an interpolated string returned from `Print.print` lowers to direct buffer
  writes.
- **In-process stress:** the two sides have separate statics and separate time accounting, and a trap names the
  side. In-memory wires can't reproduce a missing flush; that is fine only because flushing is automatic, so
  manual flush controls stay out.
- `Byte` matches need an `other =>` arm (`match` is exhaustive).

## 4.7 Open questions

1. ~~Caps checked against generators?~~ Yes, when static (§4.1) [r10].
2. ~~Tests per input?~~ Several, with `t` from `gen` and shrunk toward 1, because state leaking between tests
   is the main `@multitest` bug [r10].
