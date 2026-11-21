# Spec chapter 2: `with` contexts and `@` markers

Status: **draft v2** [proposed]. Ryan asked for one coherent design for `@`, which was doing three jobs:
configuring parameters, decorating declarations, and turning checks off [you]. This chapter splits them
into two concepts with two spellings.

**The rule:** `with` affects everything written inside it, including the calls that ask for it. `@` affects
only the declaration it is attached to.

## 2.1 Contexts

A library declares a context: a named record of compile-time constants with defaults.

```cp
context io:
    base: 0..=1 = 0
    answers: (Str, Str) = profile.answers
    precision: Int = profile.precision
    sep: Str = " "
```

Any block sets it:

```cp
with io(base: 1)                 // bare line: covers the rest of the enclosing block (here, the file)
with checks(bounds: off):        // block form
    for x in A: ...
read(base: 0)                    // an argument at one call beats every context
```

- **Precedence:** call argument > innermost `with` > outer `with` > the context's default. The call
  argument works because a `uses io` function declares a parameter defaulting to the field:
  `fn read<T>(base: 0..=1 = io.base) -> T uses io` [r10 24].
- **A bare `with` line goes at the top of a block**, before any other statement; the formatter enforces it.
  Mid-block it would silently change the lines after it [r10 Q2.2].
- **Values are compile-time constants.** Runtime values are never contexts; they are parameters or scoped
  types (chapter 1.5). Runtime contexts were tried in round 5 and brought back dynamic scope: a table built
  under modulus p silently read under another [r5].
- Standard contexts: `io` (base, answers, precision, sep), `checks` (bounds, ranges), `lint` (one field per
  lint, e.g. `float_eq: allow`), `stress` (default sizes, time budget). User libraries declare their own:
  `with mylib(mod: 998244353)`.

## 2.2 Contexts are lexical

A function body, a lambda and a tool root see the context **where they are written**, plus their own
`with`s. Nothing follows the call stack implicitly, so no function changes behaviour because of who called
it [r4]. Two failures this avoids:
- stress calls the interactor from outside every `with`, so a dynamic context would make it print 0-based
  numbers;
- any function's behaviour could depend on a `with` in some distant caller.

## 2.3 `uses`: the one hand-off

A function that *should* follow its caller declares it:

```cp
fn read<T>() -> T uses io
trait Read: fn read(r: Reader) -> Self uses io     // binds every impl
module seg uses checks                             // every function in the module
```

- The caller's context becomes an implicit compile-time argument, passed on to any `uses` callee. That is how
  `with io(base: 1)` reaches the parsing inside `read<[k]Edge<V>>`, and how `with checks(bounds: off)`
  reaches the checks inside `SegTree.apply`.
- **Where it is inferred:** only for nested functions and lambdas (one fixpoint per recursive group, never
  across a module boundary). Public functions, trait methods and modules declare it; a top-level private
  function without `uses` stays lexical [r6 11–13].
- A function value or lambda binds the context where it is written or referenced [r5 17].
- The editor marks every call that receives a context and shows the effective value at any line.

## 2.4 Cost

Contexts specialize only `uses` functions: one copy per distinct value times each generic instantiation.
Copies are cloned functions with mangled names, not C++ templates, because C++17 template parameters can't
hold floats or strings [r6 14]. `cp explain` lists every copy with its bytes, and the 48 KB size warning
counts them [r5 16]. In practice a file has one `io` value and at most a few `checks` values.

## 2.5 Checks and lints

- `with checks(bounds: off)` and `with checks(ranges: off)` remove the corresponding remaining checks in
  **release only**. Debug keeps them, so a program that passes debug has no UB in release on that input
  [r4 27].
- `with lint(float_eq: allow)` allows a lint in a scope [you]. Lints are contexts so they compose with
  everything else and need no comment syntax.
- `@unchecked` and `@checks(...)` attributes are gone.

## 2.6 Markers

`@name(args)` attaches a library-declared role to the next declaration.

| marker | on | consumed by | what it does |
|---|---|---|---|
| `@multitest` | the program | compiler | the file body runs once per test; per-test state resets |
| `@alt(f, caps)` | fn | `cp stress` | another implementation of `f` (brute or second fast); stripped from submissions |
| `@check` | fn | `cp stress` | unary output checker |
| `@gen` | fn | `cp stress` | input generator for constraints types can't state |
| `@interactor` | fn | `cp stress` | the other end of the pipes for interactive problems |
| `@grader` | fn | bundler | exports a function with the judge's signature |
| `@memo` | fn | compiler | adds a cache living as long as the declaring block (chapter 6) |
| `@depth(expr)` | fn | compiler | states the recursion depth bound |
| `@cpp("…", includes:, target:)` | fn | bundler | C++ override of an in-language body |

- Markers before the first statement of a file attach to the **program**, which is a declaration.
- The declaring library defines the role: a tool consumes it, or it wraps the declaration. A marker never
  affects code it isn't attached to.
- Tool roots (`@interactor`, `@gen`, `@check`) may capture only contexts, statics and statically sized types
  [r6 4].

`@multitest(index: case)` binds the 1-based test number to a name the marker chooses (Code Jam's
`Case #{case}: `) [p1 F8, r13 P17]. `@judge_fn` and `@mock` join the marker table.

## 2.7 `@cpp` overrides

```cp
@cpp("_pdep_u64(x, mask)", includes: "<immintrin.h>", target: "bmi2")
fn pdep(x: U64, mask: U64) -> U64:
    ...                                   // canonical body: interpreter, tracer, debug, fallback
```

The in-language body is the meaning. The C++ string is an optimization: `cp stress` checks it against the
body, and the bundler falls back to the body when the judge profile lacks the target [r7].

## 2.8 Diagnostics

- "`read` here uses `io(base: 1)` from line 1." (hover, not an error)
- "`helper` is public but reads `io` without declaring `uses io`; callers' `with io` won't reach it."
- "`@interactor` captures `n`, which is per-test state; tool roots run outside the test body."
- "`with` needs a compile-time value; `p` is read from input. For a runtime modulus use `type Z =
  Mint(mod: p)`."

## 2.8b Round-3 additions [p3]

- `@judge_fn(limit:)` and `@mock(f)` belong in the 2.6 table (chapter 4.5).
- `@multitest` reads the test count first. `@multitest(count: none)` runs until EOF. Shared input before the
  tests is written `static let x = read()` before the first per-test statement (the only place a static may
  read input). A non-static `let` that reads input there is an error.
- Typed grader signatures: `@grader fn f(domain T: 1..=1e6, book: Perm<T, T>, start: T)`. The bundler converts
  from the C signature and checks once.

## 2.9 Open questions

1. ~~Validation in contexts?~~ Only through types (literal and range types) [r10].
2. ~~Bare `with` mid-block?~~ Top of block only [r10].
3. `@grader`/`@multitest` stay markers on the program; a separate header would be a second concept [r10].
