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

## The port is the dream's shape; what still differs

`SeqFold` stores the numbers themselves (`lift: MaxSub.of`, elements `-1000..=1000` in 2
bytes beside a 4-byte own tag), takes any `Seq` for its build and `insert_many`, so the read
arrays stream into the treap, and a `?Val` tag over `?(-1000..=1000)` is one `int32_t`
(codegen packs `??range`). A tag on a run is `MaxSub.of(c).repeat(len)`, O(1) where `pow`
took O(log len) joins. At --max the port takes 0.29 s user and 42 MB against the accepted
C++'s 0.27 s and 39 MB (the v0.2 hand treap took 0.40 s and 56 MB; the limit is 64 MB).

- The dream infers `SeqFold`'s three type arguments from `op`, `lift:` and `act:`; the port
  names them (`SeqFold<MaxSub, ?Val, Val>`).
- `mirror: .reversed` (a method as a fn value) is spelled `x => x.reversed()`.
- The assign tag is `Some(read<Val>())`: the dream reads straight into the `?Val` tag type.
