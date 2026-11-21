# Cold-writer benchmark #4

Base: `main` at `726ec9b` (after Syntax Batch #8, `685655e`). The cheat-sheet
truth audit `7b72956` was not on `origin/main` when this ran, so the cheat
sheet read here is the pre-audit version. Ten standard contest problems, each
written once from `docs/one-page-cheat-sheet.md` alone, without reading
`stdlib/`, `crates/`, the corpus, or earlier benchmarks. All ten first drafts
were saved before any tool ran. Each draft then went through `cpl check`,
`cpl build`, `g++ -std=c++17 -O2 -fsanitize=address,undefined
-fno-sanitize-recover=all`, and `cpl run`. Both outputs were compared with an
independent Python reference on 40 random small inputs per problem. The
interactive problem was driven by a Python interactor over pipes, 40 games
each for `cpl run` and the binary.

## Result

- **First-try `cpl check` pass rate: 10/10**, up from 7/10 in Benches #2 and #3.
- **First-try end-to-end pass rate: 9/10.** Nine drafts built, compiled
  cleanly under ASan/UBSan, and matched the reference on all 40 cases in both
  `cpl run` and the binary. There were no wrong answers and no run-vs-build
  differences.
- **After one edit: 10/10.** The interactive guesser hung because `print`
  does not flush before a `read()`. Adding `flush()` after the query fixed it
  (40/40 games in both backends).
- `cpl lint` on the ten drafts: one warning, and it is a false positive (below).

The ten problems:

1. BFS distances on an undirected graph, with `-1` for unreachable vertices.
2. Point update and inclusive range sum with `SegTree.fold`.
3. Binary strings with no three consecutive ones: nested `[0..n + 1][0..4]Mod` DP.
4. All occurrences of a pattern, overlapping.
5. Multiset "take the smallest value >= x" with `SortedBag.first_ge` and `.first`.
6. Pop from a `MinHeap` while the top is below `k`, using `while let x = h.peek(), x < k:`.
7. Binomials mod 1e9+7 with `Comb<P>()`.
8. Shoelace twice-area of a polygon.
9. Interactive number guessing by binary search.
10. Count subsets of weights summing to `t` with `Subset<I>.full.subsets`.

## Passed first try (9)

Problems 1–8 and 10. Each draft is 4–10 lines. Things that just worked:

- **Typed reads:** `let (n, m): (Int, Int) = read()`, `let a: [I]Int = read()`
  and `let pts: [0..n](Int, Int) = read()`.
- **Domains:** `domain V = 1..=n` with `V(a)` built from 1-based input, and
  `print([d[v] ?? -1 for v in V])`.
- **Nested fill:** `var dp: [0..n + 1][0..4]Z = 0` with `dp[i + 1][j + 1] += dp[i][j]` on `Mod`.
- **Optional loops:** `if let y = bag.first_ge(x)`, `bag.first ?? -1` and
  `while let x = h.peek(), x < k:` behaved exactly as guessed.
- **Subsets:** `sum([w[v] for v in sub])` over `Subset<I>.full.subsets`, including the empty subset.

## Failed first try (1)

**Interactive guessing (problem 9).** The draft printed `? mid` and then read
the reply. Neither `cpl run` nor the release binary flushes stdout before
`read()`, so both deadlock against a real interactor. The cheat sheet only
says "`@interactor` and `@grader` cover judge protocols", and never mentions
`flush()`. The fix was one line, `flush()` after the `print`, which I found in
`proto/ports/ecnerwala/cf2152e.cp`. `yac1p3` logs the same friction
("FRICTION 46.4"), and `ceoi22p3` notes that the interpreter had no `flush()` at the time.

**Suggested fix:** flush stdout automatically before any `read()` (what
`cin.tie` does in C++), or at least document `flush()` in cheat sheet §12.
Auto-flush removes the trap entirely.

## Other surprises

1. **The release binary reads EOF as an empty value and continues.** Feeding
   problem 9 a closed stdin, `cpl run` traps with R0001 ("the input ended
   where a word was expected"). The binary instead treats the missing reply
   as `""`, keeps querying (`? 3`, `? 4`, `? 5`), and exits 0 without an
   answer. That is a silent run-vs-build divergence on malformed or truncated
   input.
2. **The cheat sheet is stale on `find_all`.** It says "free function
   `find_all(s, p)` (not `s.find_all`)". Since `685655e`, `s.find_all(p)`
   works and returns the same overlapping positions (`"abababa".find_all("aba")`
   gives `0 2 4`). The draft used the free function, which still works.
3. **The cheat sheet omits Batch #8 features that all work.** `st.fold(..)`,
   `st.fold(I(1)..)` and `st.fold(..=I(2))`; `while let x = opt, cond:`;
   nested `[0..n][0..4]Int = 0` fills; and `Set`/`Map` `.first`, `.last` and
   `.pop_first()` aren't listed. A probe confirmed each one in `cpl run` and
   the binary, with identical output.
4. **`while let x = s.pop_first(), x < 4:` drops the first failing element.**
   On `{1, 3, 5}` it prints 1 and 3, then pops 5, fails the guard and leaves
   `s.len == 0`. That's consistent with how the guard is defined, but it's
   an easy silent loss. The safe spelling is `peek`/`first` in the guard:
   `while let x = h.peek(), x < k:` (problem 6). The cheat sheet should show
   the peek form.
5. **False-positive L0203.** `let a: [I]Int = read()` followed by
   `var st = SegTree<I, Int>(+, a)` warns "this binding hides the top-level
   `a`", pointing at the argument use. Nothing is rebound there.

## Trend

| Bench | First-try check | First-try end-to-end | After one edit |
|---|---|---|---|
| #2 | 7/10 | 7/10 (samples only) | — |
| #3 | 7/10 | 7/10 | 9/10 |
| #4 | 10/10 | 9/10 | 10/10 |
