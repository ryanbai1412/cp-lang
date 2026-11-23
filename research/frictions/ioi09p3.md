# ioi09p3 friction

## A `let`-bound lambda used only inside another lambda types its parameter as `Int`

- **Where:** `corpus/problems/ioi09p3/spec.cp` wants `let key = c => (desc(score[c]), desc(solved[c].sum()), c)`,
  used only in `C.count(c => key(c) < key(p))`. The parameter defaults to `Int`, so `score[c]` is
  E0301 "indexed by `C`, but the index is a `Int`"; R05.10.6 says a lambda infers its parameter
  types from where it is used. The spec writes `(c: C)` meanwhile.
- **Repro:** `domain C = 1..=3; let s = [C] c => 0; let key = c => s[c]; print(C.count(c => key(c) > 0))`
  is E0301; the same `key` with a top-level use, `print(key(C(1)))`, checks clean.
- **Goal it hurt:** express (an annotation the uses already fix).
- **Routed:** compiler worklog.
