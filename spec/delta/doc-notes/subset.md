# Doc notes: `std.subset`

Choices made where the spec is silent, and contradictions found, while bringing
`spec/std/subset.cp` to the doc standard (`spec/std/README.md`).

- `Subset.subsets`: chose the submask walk `g = (g - 1) & s` exactly, so the
  order is decreasing as words; the spec only fixes `s` first and `{}` last
  (lib/search-dp.md, decisions.md), and this is the one walk with that shape.
- `Subset.supersets`: chose `g = (g + 1) | s`, increasing as words, `s` first and
  `B` last; the spec names the generator but not its order. Mirrors `subsets`.
- `Subset.with` / `without`: adding a member or removing a non-member returns
  `self` unchanged (set semantics, as `Set.add`/`remove` never trap).
- `Subset.first`: needs a `ties:` label by the audit rule; members are distinct,
  so it says there are no ties and the least in `B`'s order wins.
- Contradiction: `subset.cp` says `B` has at most 64 members (one `U64`);
  lib/search-dp.md §Mask DP says at most 30. Left the stub; one should change.
- Contradiction: decision 62 says `Subset<D>` has `empty` and `full` "as `@get`"
  and decisions 18.3 and 19.x write `Subset<D>.empty` / `.full` bare; the stub declares
  `pub fn empty()` and `pub fn full()`, which need `()`. Left as is.
- Contradiction: decision 62 cites `Subset<D>.set(i, on)` and decision 16.8
  `Subset<D>.shifted(k)`; neither exists in `subset.cp` (Bitset has both).
- Gap: decision 18.3 says `for s in Subset<D>` walks all masks in ascending
  integer order; the stub says `B.subsets` is the domain but nothing states its
  order. Not documented here because it is a domain fact, not a fn.
