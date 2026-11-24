# Friction Report: ccc02s4hard

The v0.2 port hand-rolled a lazy segment tree: five parallel `List<Int>` arrays sized to a power
of two, `pull`/`push`/`apply_tag`/`build`/`st_upd`/`st_query` written out, an `inf` sentinel for
empty nodes and a `-1` "no tag" value, global `q_cost`/`q_at` for query results, an `inf`
sentinel at the bottom of the monotone stack, and push loops to read the people. Now people are
one typed read, the tree is std `SegTree<Cut, Cell, ?Int>` with an `Action` whose tag assigns
the slowest time, empty cells are `None`, ties to the smallest cut ride in `(val, at)` order,
and the stack holds only real people.

## Optional min with an arg needs a hand-written `lower`
- **Where:** `fn lower(x: ?Best, y: ?Best) -> ?Best`, the identity-carrying min of `Best`.
- **Ideal:** `Monoid.min.opt()` (or `min` lifted over `?T` with `None` as the identity).

## An assign tag needs its own `then`
- **Where:** `fn then(new: ?Int, old: ?Int)`, the newest-wins composition.
- **Ideal:** a stock `Action.assign(apply: ...)` beside the stock `add`.

## Dream
```cp
let people: [n](name: Str, time: 1..=1e9) = read()
domain Cut = 0..=n
var st = SegTree<Cut, (low: Best, total: Best), Int>(min.opt(), act: assign((x, c) => (c.low, c.low + x)))
var next: [Cut]Cut = Cut.last!
for i in Cut.before_last.rev:
    st.apply(i.next..=taller.after(i) ?? Cut.last!, people[i].time)
    let best = st.fold(i.next..=min(i + m, Cut.last!)).total
    next[i] = best.at
    st.set(i, (low: (best.val, i), total: None))
print("Total Time:", st.get(Cut.first!).low.val)
for (a, b) in Cut.first!.follow(next): print(people[a..b].names)
```

- Tokens: port 637, original 1718
- Max input (`gen.py --max`): port 0.07 s, 24120 KB, original 0.09 s, 18560 KB
