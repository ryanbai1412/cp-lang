# Friction report: dmpg16g2

- `Letter.subsets` on `enum Letter = A | B | C | D | E` is E0204 "`Letter` declares no domain
  `subsets`", and an integer domain's `B.subsets` used as a value (`B.subsets.sum(of:)`,
  `.filter`, `.last!`) is E0501 in lowering; both already routed (compiler worklog, idioms
  request 2026-11-22). spec.cp uses `domain Letter = [5]` and plain `for s in Letter.subsets`
  loops with `if ch in s`.
- `last.buckets()` on `[Col, Letter]?Row` is E0302: `?Row` is not a domain, as `buckets` requires
  (`K: Domain`), so the old spec.cp was wrong under the spec. spec.cp pushes into
  `[Row]List<(Col, Letter)>` instead.
- Side finding: for an enum `L`, `print(L.at(1))` reports `L.at(1)` as `?L` but `L.at(1)!` is
  E0302 "`!` takes the value out of an optional, but this is a `L`".
