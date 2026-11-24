# Friction report: noi05p2

## Dream

```cp
let assign = Action<MaxSub, Int>.new(apply: (c, x, len) => MaxSub.repeat(c, len), ..)
var seq = SeqFold<MaxSub, Int>(MaxSub.join, [MaxSub.of(a) for a in init], act: assign)
match op:
    "INSERT" => seq.insert_many(posi, [MaxSub.of(x) for x in read(tot)])
    "DELETE" => seq.erase_range(posi - 1..posi - 1 + tot)
    "MAKE-SAME" => seq.apply(posi - 1..posi - 1 + tot, c)
    "REVERSE" => seq.reverse(posi - 1..posi - 1 + tot)
    "GET-SUM" => print(seq.fold(posi - 1..posi - 1 + tot).total)
    _ => print(seq.total.best)
```

## Blocked: no std sequence with fold, erase and bulk insert

`std.lazy.SeqFold` folds, applies, reverses and inserts one element at a time, but has no
`erase_range`, `split_at` or `splice`. `LazyTreap` has `erase_range`, `split_at` and
`splice_back`, but no `fold`. The port therefore keeps the v0.2 hand treap: twelve parallel
`List<Int>` node fields, a free list and a sentinel node 0. The repro went to the stdlib
owner, and the port moves to the dream once `SeqFold` gains `erase_range` and a bulk insert.
