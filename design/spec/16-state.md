# 16. State and value semantics

**Layer:** core C10, C15 (program shape).

Status: [proposed].

**Why.** Value semantics with no move checker: C++'s copy behaviour, minus aliasing bugs. Bindings above the wrap line are the one channel across tests. Core, except `List` growth, which is library over `alloc`.

## 16.1 Invalidation

Only length-changing operations on a `List` (`push`, `pop`, `resize`, `clear`) invalidate, and only views and
slices of that same `List`. Element writes never do.
- When the access paths are syntactically the same (same root variable, same index expressions, no write
  in between), it is a compile error. `let r = adj[u]` is a slice, not an alias of the `List` [r10 26].
- Otherwise debug checks a per-`List` generation count: `adj[w].push` while iterating `adj[u]` traps only if
  `w == u`.
- Views and slices can't be stored in struct fields or statics. Stdlib types (`Graph`, `Buckets`) store
  offsets and build slices on access; user libraries can do the same [r6 17].
- A slice is (pointer, length) into the element buffer, never a borrowed `List` header [r6 15].

## 16.2 Bindings and state

- `let` binds once; `var` is mutable. Every `var` is initialized at its declaration (`var a: [V]Int = 0`
  fills). There is no uninitialized `var` and no `++`.
- A `let` may shadow an earlier binding in the same block, except a name captured by a hoisted function
  (compile error) [r11 N12]. In an un-annotated `var`, a range type infers `Int`; every other type
  (positions, fixed widths, structs) infers itself, so `var total = 1; total += x` never trips a contract; a `let` keeps the computed range [r11 N6].
- **Bindings above a `--- with multitest() ---` line** are the only state shared across tests (16.3).
  Everything below it is per test.
- **Heap values copy on assignment** [p1 Q3, proposed default]: `var out = s` copies a `List`, `Str` or `Map`,
  as C++ does. There is no move checker. A lint and `cp explain` flag copies of large values inside loops;
  pass by `let` parameter to avoid them (parameters never copy). A temporary moves instead of copying [r13 P5].

## 16.3 Program shape [provisional, deferred design question, CORE-ALIGN area 2]

> **Provisional:** `--- multitest ---` / `--- with multitest() ---` is currently accepted by the parser as an alias for `@!multitest`, but program shape remains a deferred design question. Keep `@!multitest` as the canonical spelling in corpus ports and do not codemod ports onto wrap-lines until program shape settles.

**What.** The entry file's body is the program, run once. A **wrap line** `--- with f(args) ---` passes
the rest of its block to `f` as a closure, so `--- with multitest() ---` makes everything below it the
per-test body without indenting it.
```cp
with io(base: 1)
let fact = Z.factorials(2e5)                  // above the line: per program

--- with multitest() ---
domain V = read(1..=2e5)                      // below: per test, unindented
let adj = adjacency(read<[V.count - 1](V, V)>())

fn deepest(v: V, p: ?V) -> Int:              // sees V and adj
    …

if adj[V.first].is_empty:
    print(-1)
    return                                    // returns from the per-test closure: the next test
print(deepest(V.first, none))
```

**Rules.**
1. **Wrap lines are sugar.** `--- with f(a…) ---` followed by the statements `R` that remain in its block
   means `f(a…, () => R)`. `--- with p <- f(a…) ---` means `f(a…, p => R)`. The closure is appended as the
   last positional argument; `f(a…)` must be a call. A wrap line is legal wherever a statement is, and
   ends its block: nothing after it is outside the closure.
2. **`R` is an ordinary closure body.** Its bindings, `domain`s and functions are locals of the closure.
   `return e` in `R` returns from the closure, so the closure's type is `() => T` with `T` inferred from its
   returns (unit when there are none). `break`/`continue` can't cross the closure. `R` may read and
   mutate bindings above the line (a closure captures by reference, 14.2).
3. **Functions are hoisted within their block** (above the line: the file; below: `R`), so they may come
   before or after their callers. A call is a compile error if a binding the callee captures, directly or
   through its callees, isn't definitely initialized at the call; the error names both [2impl Q8].
   Functions above the line can't see bindings below it.
4. **The entry file body runs once, top to bottom.** Its top-level `let`/`var` bindings are per program;
   top-level functions capture them like any nested function (14.1). `return` at the top level ends the
   program. Without a wrap line, the whole file is one test.
5. **Lifetimes fall out of scope.** A fresh type declared below the line is scoped to one call of the
   closure, so its values can't cross tests (12.2). A `@memo` cache lives as long as its declaring block:
   per test below the line, per program above it. A per-program value can't have a fresh type declared
   below the line.
6. **Library files** (every file but the entry file) hold declarations and top-level `let`/`var` bindings
   only: no statements and no wrap lines. A library binding is initialized on first use, so unused library
   state costs nothing; its initializer can't read input or call `gen`. A file whose only entry point is
   a `@grader` function has no top-level statements.
7. **Tool roots** (`@gen`, `@check`, `@interactor`) capture nothing below a wrap line; one declared below is
   a compile error naming the captured binding. `@alt(program)` replaces the innermost top-level wrap
   line's closure when there is one, and the whole file body otherwise (30.1).
8. **Lowering.** The closure's locals become fields of a generated per-test struct whose methods are `R`'s
   functions (so recursion needs no `std::function`), or plain locals when no function captures them.
   Per-program bindings become locals of `main`, or globals when a function captures them. `multitest`
   is generic over the closure type, so the call inlines; nothing is heap-allocated.

**Multitest is library code.** The standard wrappers:
```cp
enum Count = read | eof | n(Int)
fn multitest<T: Print>(count: Count = .read, prefix: ?(Int => Str) = none, body: () => T) uses io
fn tests(count: Count = .read, body: Int => ()) uses io        // passes the 1-based case number
```
`multitest` prints `body`'s result when `T` isn't unit. Formats:

| input or output | wrap line |
|---|---|
| `t`, then t tests | `--- with multitest() ---` |
| tests until EOF | `--- with multitest(count: .eof) ---` |
| `Case #i: ` before each answer | `--- with multitest(prefix: i => "Case #{i}: ") ---` |
| need the case number in the body | `--- with case <- tests() ---` |
| return each test's answer instead of printing it | `--- with multitest() ---`, then `return ans` |
| anything else | a function in your file or library, e.g. `fn pairs(body: (Int, Int) => ()) = repeat read(): body(read(), read())`, then `--- with (n, k) <- pairs() ---` |

**Why.** Long solutions stay flat: per-test state is declared at the top level of `R`, and every chunk is a
function in `R` that sees it, with no indentation and no parameter threading. Per-program state needs no
keyword (it is above the line), and multitest formats need no decorator (they are functions). The only
new syntax is the wrap line, and it has a one-line desugaring.

**Cost.** Nothing at run time (rule 8).

**Rejected.** An explicit `fn solve():` (one indentation level over the whole solution); an implicit
`solve` with `main` outside it and `once let` for per-program values (two special names, a keyword, and
`main` can't exist in a library); per-program state as `main`'s locals (no `main` in a library);
`--- solve`/`--- brute` named sections (brute and gen aren't special); test-scoped globals assigned inside
`solve`; a state struct with outside methods (every field written twice); `@multitest` on the file body (a
decorator on something that isn't a declaration); `static`.
