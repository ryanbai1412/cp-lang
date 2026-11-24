# ucrpc21e — Feeding Friendsy

## `flat_map` over a domain loses a tuple element type
- **Where:** `port.cp`, building `changes`.
- **What felt wrong:** `List(Mouth.flat_map(mouth => windows[mouth].flat_map(w => [(at: w.a, mouth:, d: worth(w)), ...]))).sorted()`
  types its elements as `Int` (E0203 on `c.at`), so the events are pushed in a loop instead.
- **General fix:** infer `flat_map`'s element type from the closure's list literal (sent to Core).
- **Goal it hurts:** express.
- **Severity:** low.
