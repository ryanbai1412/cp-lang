# 17. Contexts

**Layer:** core C11. `io`, `checks`, `lint` are library-declared contexts.

Status: [proposed].

**Why.** One mechanism for file- or block-wide configuration (`io(base: 1)`, `checks(bounds: off)`, `lint(float_eq: allow)`) instead of a flag per feature. The mechanism (`with`, `uses`, lexical scoping, compile-time values) is core; every context is declared by a library like any type.

## 17.1 Contexts

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
  types (chapter 12.3). Runtime contexts were tried in round 5 and brought back dynamic scope: a table built
  under modulus p silently read under another [r5].
- `with ctx(…)` (a context) and `--- with f(…) ---` (a wrap line, 16.3) share the keyword; whether contexts
  become `set f(name: v)` defaults so `with` means only wrapping is CORE-ALIGN area 3 [p4].
- Standard contexts: `io` (base, answers, precision, sep), `checks` (bounds, ranges), `lint` (one field per
  lint, e.g. `float_eq: allow`), `stress` (default sizes, time budget). User libraries declare their own:
  `with mylib(mod: 998244353)`.

## 17.2 Contexts are lexical

A function body, a lambda and a tool root see the context **where they are written**, plus their own
`with`s. Nothing follows the call stack implicitly, so no function changes behaviour because of who called
it [r4]. Two failures this avoids:
- stress calls the interactor from outside every `with`, so a dynamic context would make it print 0-based
  numbers;
- any function's behaviour could depend on a `with` in some distant caller.

## 17.3 `uses`: the one hand-off

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

## 17.4 Cost

Contexts specialize only `uses` functions: one copy per distinct value times each generic instantiation.
Copies are cloned functions with mangled names, not C++ templates, because C++17 template parameters can't
hold floats or strings [r6 14]. `cp explain` lists every copy with its bytes, and the 48 KB size warning
counts them [r5 16]. In practice a file has one `io` value and at most a few `checks` values.

## 17.5 Diagnostics

- "`read` here uses `io(base: 1)` from line 1." (hover, not an error)
- "`helper` is public but reads `io` without declaring `uses io`; callers' `with io` won't reach it."
- "`@interactor` captures `n`, which is per-test state; tool roots run outside the test body."
- "`with` needs a compile-time value; `p` is read from input. For a runtime modulus use `type Z =
  Mint(mod: p)`."
