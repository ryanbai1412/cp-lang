# Friction report: noi03p2

## Dream

```cp
var text = Rope<Char>()
'Insert' => text.insert(cursor, chunk)
'Delete' => text.erase(cursor..cursor + n)
'Get'    => print(text[cursor..cursor + n])
```

## Port

Already at bar: with at most 4000 edits a piece table (a `List<Str>` cut where edits land,
plus piece start offsets searched with `bsearch`) is the rethought design, not a translation
of the original's rope.

## Remaining friction

- std has no growable text/sequence with insert, range erase and slice in O(log n):
  `LazyTreap` has `insert_at`/`erase_range`/`split_at` but needs `apply` and `compose`
  closures even with no tags, and `Rope` is fixed-size over a domain.
- `List` has no range erase, so a delete removes pieces one by one.
- The command count has no stated bound, so `t` stays `Int`.

## Verification

The v0.2 snapshot no longer builds (`partition_point` is gone), so the port is compared with
the accepted C++: 30 random cases agree. Corpus tests (6) pass; the max case runs in 0.11 s.
