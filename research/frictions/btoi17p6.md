# Friction report: btoi17p6

- `Rooted.from_parents(read<[V.drop(1)]V>())`, the "parents of 1..n-1" shape the
  `Rooted.from_parents` doc names, is E0504 "fresh type made from values"; already
  routed as compiler worklog B3 and stdlib worklog H01. spec.cp reads `[n - 1]V`
  and builds `[V]?V` with `[V] v => if v == V.first!: None else: Some(up[v.ord - 1])`.
