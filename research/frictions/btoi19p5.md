# btoi19p5 frictions

## Dream

```cp
let s: Str<1..=3000> = read()
let t: Str<1..=3000> = read()
let best = [t, t.rev].map_indexed((u, flip) => longest(u, flip)).max(by: b => b.len)!
```

## Reversed views

`prefix_matches` needs an `Indexed`, so the reversed strings are copied with `List(u.rev)` on
every cut; a reversed view that is itself `Indexed` would drop the copies. The flipped answer
maps its start in reversed t back by hand.
