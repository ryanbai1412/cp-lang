# std.hashmap doc notes

Choices made where the spec was silent, and contradictions found, while
bringing `spec/std/hashmap.cp` to the doc standard (`spec/std/README.md`).

## FlatMap

- `FlatMap.new(default_val, cap)`: `default_val` only fills empty value
  slots; no public method returns it. Contradiction: `lib/containers.md`
  2.8 has `new(cap: Int = 16)` with no default value. The stub is kept.
- `FlatMap.new` / `reserve`: the table is a power of two at least twice
  `cap` (the load factor <= 0.5 of `lib/containers.md` 1.7), so `cap`
  entries fit without a rehash; `cap < 1` gives the smallest table.
- `FlatMap.items` / `keys` / `values`: order is unspecified slot order
  (depends on the per-map salt), but the three agree while the map is
  unchanged. Examples sort before comparing. If "everything is
  deterministic" (AGENTS.md goal 4) must cover hash order, the salt has to
  be seeded per run, which the spec does not say.
- `FlatMap.absorb(other, join)`: for a shared key the result is
  `join(self's, other's)`, or other's value when `join` is `None`,
  whichever side the small-to-large merge drains. Without this rule the
  result would depend on the sizes.
- Contradiction: `lib/containers.md` 2.8 and decisions.md 44.5 say
  backward-shift deletion ("tombstones never degrade performance"), but
  the stub's fields and comment use a tombstone state (`2 = tombstone`).
  Not observable through the API; left alone.
- Contradiction: `lib/containers.md` 2.8 lists `each(self) -> yields (key:
  Int, val: V)`; the stub has `items` / `keys` / `values` getters and no
  `each`, and it mentions `FlatSet`, which no stub declares.

## Counter

- Invariant chosen: no stored entry equals `init`. `add` already removed a
  key whose value returns to `init`; `set` and `absorb` now do the same, so
  `len` is "keys whose value differs from `init`" and `contains(k)` is
  `get(k) != init`. This makes `len` the distinct count that
  `lib/sequence-patterns.md` relies on (`c.distinct <= k`).
- `Counter.absorb`: stored values are added; a key stored on one side keeps
  its value (absent keys contribute nothing, not `init`), so the result does
  not depend on which side the small-to-large merge drains. With `init`
  `0` this is plain addition of counts.
- `Counter.total`: sums stored entries only; empty gives `V.zero`.
- `Counter.most_common(k)`: ties in ascending key order; `k >= len` returns
  everything; `k < 0` traps. Ascending key order needs `K: Ord`, which the
  signature does not state; insertion order (Python's choice) would need
  extra bookkeeping.
- `Counter.add` / `c[k] += d`: documented as the same operation.
  decisions.md (missing-key reads) says `c[k]` reads the default for
  `Counter` without inserting, which `get` matches.
- Contradiction: `lib/sequence-patterns.md` (Track table, `maximal`
  example, "count subarrays with sum K") and `core/07-sequences.md`
  (`s.tally(by: k) -> Counter<K>`) use a one-parameter `Counter<K>` with
  `.distinct`, `push` and `pop`; decisions.md (around "`Counter<K>` and
  `counts(xs)`") also says `Counter<K>`, and another entry moves `Counter`
  to `examples/local-lib/`. The stub is `Counter<K, V>` with `init`, no
  `distinct`, `push` or `pop`, and lives in `std.hashmap`. Needs one
  decision: which name, arity and module, and whether it implements
  `Track`.
