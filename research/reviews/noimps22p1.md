# noimps22p1

Score: 4/5
Dream: 4/5

Cold review on main 6a989896d: port, dream and friction report, not the C++. The port reads like the dream: `line.runs(by: is_ident)` and a recursive expand that skips active names. The remaining gaps (`strip_prefix`, the header count read as a line) are in the friction; this review adds the `cpl run` for `strip_prefix`.

## Checklist failures

- Line 27 removes `run.to_str()` where line 23 already bound it as `word` (idiomatic-code/R4).
- The header has no `// Bounds:` line for `n` (read on line 11).

## Owed rewrites

- Drop the stale friction "A recursive call with one more set element needs a copy": the port now inserts and removes on one `active` set (lines 25-27).

## Language gaps

- `Str.strip_prefix`: friction (run added: `error[E0203]: a `Str` has no method `strip_prefix``).
- Typed count then verbatim lines: friction.

## Dream notes

- Declares no `macros` map, and `expand(line, [:])` passes a map literal where the signature asks for `Set<Str>`.
- `;`-chained statements on one line are not house style.
