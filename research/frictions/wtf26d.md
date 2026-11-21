# Friction: `wtf26d`

## No `p.inverse()` for permutations between two domains (`[Pos]Val -> [Val]Pos`)
- **Status:** fixed: `std.seq.inverse<I, V>(self: [I]V) -> [V]I` supports method syntax `p.inverse()` across distinct domains; `wtf26d.cp` now writes `let inv_p = p.inverse()`.
- **Where:** `wtf26d.cp` lines 13–14 (`dream/wtf26d.cp` line 15):
  ```cp
  var inv_p: [Val]Pos = Pos.first
  for i in Pos: inv_p[p[i]] = i
  ```
- **What felt wrong:** Inverting a bijection `[Pos]Val` still needs a dummy initial value (`Pos.first`) and a manual loop.
- **Goal it hurts:** express, hard-to-get-wrong
- **Ideal code (`dream/wtf26d.cp` line 15):**
  ```cp
  let inv_p: [Val]Pos = p.inverse()
  ```
- **General fix:** Add `inverse` for bijections / `Perm` in `std.seq` (or `Perm<D>`).
- **Severity:** low

## No `List.pop_while(pred)` and no `Option.filter` / `Option.map` chaining
- **Where:** `wtf26d.cp` lines 35, 38, 61–64 (`dream/wtf26d.cp` lines 31, 34, 60):
  ```cp
  while let top = items.last, pos < top.max_p: items.pop()
  ```
- **What felt wrong:** Draining a monotone stack repeats peek + predicate + `pop()`, and checking `pref_min[cap]` against `pos` needs nested `if let w = pref_min[cap]:` instead of `pref_min[cap].map(w => pos <= w) ?? true`.
- **Goal it hurts:** express
- **Ideal code (`dream/wtf26d.cp` lines 31, 60):**
  ```cp
  items.pop_while(c => pos < c.max_p)
  if pref_min[cap].map(w => pos <= w) ?? true: return false
  ```
- **General fix:** Add `List.pop_while` and `?T.filter` / `?T.map` combinators.
- **Severity:** low

## Measurements
- **Contestant-written tokens:** 858 (port) vs 874 (dream, **0.98x**).
- **Emitted C++ bytes:** ~20.1 KB (31% of 64 KB limit; grew from 16.1 KB after replacing raw `Int` sentinels with `Pos`/`Val` domains and `CondStack`).
- **Gate:** `gate.sh wtf26d 150` PASS.
