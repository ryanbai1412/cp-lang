# Friction report: noi07p1

## No `Bool.then` for "Some when true"

- **Where:** `[V, V] (s, t) => if s == t: Some((d: 0, ways: 1)) else: None`.
- **Ideal code (`dream.cp`):** `(s == t).then((d: 0, ways: 1))`.
- **What happened:** E0203 "a `Bool` has no method `then`".
- **General fix:** std `Bool.then(x) -> ?T`, one spelling for a conditional optional.
- **Severity:** low.

## No ordered pairs of a domain

- **Where:** `[(s, t) for s in V for t in V if s != t and s != v and t != v]`.
- **Ideal code (`dream.cp`):** `V.pairs_ordered().filter((s, t) => s != v and t != v)`.
- **What happened:** E0203 "a `domain V` has no method `pairs_ordered`". The comprehension works
  but spells out `s != t` again.
- **General fix:** `pairs_ordered` / `pairs` on a domain, the same as on a list.
- **Goal it hurts:** express. **Severity:** low.

## Measurements

Contestant-written tokens: port 316, original 444 (`corpus/audit.py` count). Emitted C++: 14608 bytes (`cpl build`, judge `generic`).
