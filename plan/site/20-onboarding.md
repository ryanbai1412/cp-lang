# Onboarding: getting started, quickstarts, tour

Status: proposed. Three entry pages under `/learn`, all runnable in the
browser, none needing an install.

## `/learn/start`: getting started (5 minutes)

For someone who has never seen CP Lang and wants one working program.

1. **What CP Lang is**, in three sentences: a contest language, compiles
   to one C++17 file, the browser runs the same compiler (WASM build of
   `crates/cpl-wasm`, *verified*).
2. **Your first program**: an embedded editor with
   ```
   let n: 1..=2e5 = read()
   domain V = 1..=n
   let a: [V]Int = read()
   print(a.sum())
   ```
   (`site/examples/sum.cp`, *verified* to run) and input `5 / 3 1 4 1 5`.
   The reader presses Run, sees `14`, then clicks *C++17* and sees the
   generated file with the source quoted at the top.
3. **Break it on purpose**: change `5` to `6` in the input; the
   validation trap names the line and the rule. Change `a.sum()` to
   `a.summ()`; the error suggests `sum`.
4. **Where next**: three cards: the quickstart for your language, the
   course, the tour.
5. **Run it locally later**: one line linking Setup (`60-tools-setup.md`).

## `/learn/from/<lang>`: four quickstarts

One page each for C++, Python, Java and Rust. Each takes about 15
minutes and is built on the same six problems, so the pages stay
parallel and share their tests. Every problem has: the statement (three
lines), how you'd write it in your language (a short, idiomatic
snippet, shown collapsed), the CP Lang solution in an editor with *Run*
and *Submit* (judged against hidden tests like a course problem), and
"what's different" notes written for that language.

### The six problems (original, written for these pages)

1. **Sum of a list.** Read `n` and `n` numbers, print the sum.
   Teaches `read`, ranges as input contracts, `print`.
2. **Pairs with a target sum.** Sorted array, count pairs summing to
   `x`. Teaches `var`, `while`, slices and two pointers.
3. **Nearest exit on a grid.** BFS from `S`, print the distance to the
   nearest `E` or `-1`. Teaches `Grid`, domains, `?Int` distances and
   `??`.
4. **Count paths in a DAG mod 1e9+7.** Teaches functions, `Mod<P>`, and
   memoized recursion (`@cache` once it runs; until then the form that
   runs, with a `checks` note showing the 0.3 form).
5. **Range minimum queries with updates.** Teaches the library and
   operation values: `SegTree` with `op: min`.
6. **Several test cases.** Problem 3 with `t` test cases. Teaches
   `@!multitest` and why state can't leak.

### What each page's notes say

- **C++**: `int` vs range types and overflow proofs; `INF`/`-1`
  sentinels vs `?T`; `vector<vector<int>> adj` vs `Graph` and domains;
  global arrays and multitest leakage; what the generated C++ looks
  like and how to submit it; "rethink, don't translate" (from
  `site/content/cpp-guide.md`).
- **Python**: indentation and `:` feel familiar; types are required at
  `read` and in signatures, inferred elsewhere; speed (this compiles to
  C++, no TLE from the interpreter); recursion depth isn't 1000; no
  `None` checks at runtime because `?T` is checked at compile time.
- **Java**: no classes or `new`; structs and enums are values; no boxing
  (`?Int` is usually the size of an `Int`); `Scanner`/`BufferedReader`
  boilerplate goes away; generics without erasure.
- **Rust**: ownership feels familiar but there are no lifetimes in
  signatures except `-> lends T`; no `&`/`&mut` (parameters are lent,
  `mut` marks mutation at both ends); no `unwrap` chains (`??`, `!`
  traps with a source-level report); integer widths come from ranges.

Each claim in these notes cites the spec chapter it rests on, and each
"how you'd write it" snippet in the other language is compiled or run
by the gate too (g++, python3, javac, rustc), so the comparison never
rots.

### End of each quickstart

"You've written six programs. Next: the course (problems with
verdicts), or set up your editor." Plus the cheat sheet link.

## `/learn/tour`: the language tour (one sitting, about 40 minutes)

One long page that covers the whole language at reading speed, every
example runnable. It replaces `site/content/tour.md` and follows
`spec/core` in order, one section per chapter, each section 2 to 4
examples:

1. Values, ranges and `let`/`var` (chapter 2, 3)
2. `?T`, `None`, `Some`, `??`, `!` (2.4)
3. Domains, positions, cuts and ranges (4, 4b)
4. Arrays, lists, maps and slices (2.8, 7)
5. Functions, nested functions, lambdas, generators, decorators (6)
6. Ownership: owned and lent values, `mut`, `.clone()` (5)
7. Structs, enums and `match` (2.6, 2.7)
8. Traits, generics and operation values (8, `lib/ops-ranges.md`)
9. Contexts and checks: checked vs release builds (9)
10. Input, output and interaction (10)
11. Modules, the prelude and libraries (11)

Each section ends with a link to the reference chapter it summarizes.
Snippets that only check today carry the `checks` badge and run once the
implementation catches up; the gate tracks this.
