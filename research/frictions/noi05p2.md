# Friction report: noi05p2

## Dream

```cp
var seq = SeqFold(+, read<[Init]-1000..=1000>(), lift: MaxSub.of, act: assign, mirror: .reversed)
match op:
    "INSERT" => seq.insert_many(posi, read<[tot]-1000..=1000>())
    "DELETE" => seq.erase_range(r)
    "MAKE-SAME" => seq.apply(r, read())
    "REVERSE" => seq.reverse(r)
    "GET-SUM" => print(seq.fold(r).total)
    _ => print(seq.total.best)
```

## std SeqFold now has the edits; it is still heavier than a hand treap

`SeqFold` gained `insert_many` (an O(k) Cartesian-tree build), `erase_range` (erased nodes
are reused), `split_at`, `splice`, 4-byte link, priority and size fields, and `mirror:`
(`MaxSub.reversed`), which keeps one fold per node instead of a forward and a backward one.
A port on them is the dream's shape and agrees with the v0.2 port and the accepted C++ on 30
random cases each, but at --max it takes 0.40 s and 66 MB, against the accepted C++'s 0.28 s
and 39 MB and the v0.2 hand treap's 0.40 s and 56 MB (the limit is 64 MB). So the port keeps
the v0.2 hand treap for now. What is left, each general:

- Each node stores its element as a full `MaxSub` (40 bytes) beside its fold, where the
  accepted C++ stores one int: an element type apart from the summary (`lift:`) is the fix.
- A generator or `.map` passed where a `Seq` is taken is collected into a `std::vector`
  first, so building from the input costs a 20 MB `List<MaxSub>` (the port pushes one by
  one instead).
- A `?F` tag over `F = ?Int` is a 24-byte `optional<OptInt>`, even for a range-typed `F`.
