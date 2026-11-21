# 31. Interaction

**Layer:** library. `Wire<Q>` and `Limited` are library types over `Read`/`Print`; `@interactor` is a role marker. Only the bundler's knowledge of what to strip is tooling. Status: [proposed].

## 31.1 Interaction

```cp
let judge = Wire(error_reply: -1)
fn neighbour(v: V) -> V = judge.ask("? {v}")         // exits on the raw error reply, so `V`, not `?V`
judge.say("! {answer}")
```

- An `@interactor` is an ordinary program run by `cp stress` on the other end of the pipes. It sees the
  program-level context, not per-test state.
- The stdlib `Wire` owns error replies, query limits and flushing. An interactive error reply is compared on the raw
  token, before base conversion.
- **Flush rule:** see 31.3: constructing a `Wire` switches stdout to interactive mode at runtime [2impl Q75].
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
  - A limit shared across tests is an untyped `Wire` `var` above the wrap line with a typed view per test (31.2).
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

## 31.2 Round-15 interaction fixes [p3]

- **One interactor signature:** `@interactor fn judge(tests: List<Case>, wire: Wire)`. Per-case typing and limits
  use a view: `let w = wire.typed<Query<V>>(limit: [asks: V.count])`. The raw stdio is one resource.
- **Shared limits across tests:** `var raw = Wire(error_reply: -1, limit: …)` above the wrap line (untyped, so legal as
  per-program state), with `raw.typed<Query<V>>()` per test. Limits live on the raw wire.
- **Limits are a `[Kind]Int` literal:** `limit: [asks: V.count]`.
- **Handles:** `Wire` counts, so it is a `var`. The solution calls `judge.query(.ask(v))`. A leading-dot
  variant `.ask(v)` is allowed wherever the expected type is a known enum.
- **No per-message allocation:** an interpolated string returned from `Print.print` lowers to direct buffer
  writes.
- **In-process stress:** the two sides have separate statics and separate time accounting, and a trap names the
  side. In-memory wires can't reproduce a missing flush; that is fine only because flushing is automatic, so
  manual flush controls stay out.
- `Byte` matches need an `other =>` arm (`match` is exhaustive).

## 31.3 [p4] flushing is library
Replaces the compile-time flush rule (old 4.4). Constructing a `Wire` switches stdout to interactive mode at
runtime; in that mode every `read` and every exit (including `fail`) flushes first. Programs that never
construct a `Wire` never flush early. The cost is one predictable branch per read. No reachability analysis,
and a user library can build its own `Wire` with the same power.
