# Friction Report: cco16p6

The v0.2 port already used std `SegTree` with `right_while`; it kept a `Node` struct, the
names `kv`, `arr`, `mx`, `set_slot`, an index loop over the votes, and wrote the last bucket as
`Slot(n + 1)`. Now the monoid is the named tuple `Load = (count, sum)`, the votes are walked by
`.items`, the overflow bucket is the named `rich = Bucket.last!`, and the price convention
(`b - base + 1`) is stated once. Adds tests.toml.

## The rich pirate breaks the bucket encoding
- **Where:** `st.set(rich, (count: 1, sum: keep + base))`: one voter whose price lies past every
  bucket rides in the last bucket with a sum that is not `index * count`.
- **Ideal:** prices as keys of an ordered multiset with an origin shift (`OffsetMultiset`), so
  "the cheapest `need` prices summed" is one call and no bucket is special.

## Dream
```cp
var prices = OffsetMultiset<Int>()  // every price +1 per pirate in O(1)
for (i, v) in votes.items:
    let cost = prices.smallest_sum(v - 1)
    if cost <= k:
        prices.keep_smallest(v - 1).raise_to(1)
        prices.add(k - cost + 1)
        print(k - cost)
    else:
        prices.add(1)
        print(-1)
    prices.shift(1)
```

- Tokens: port 443, original 1154
- Max input (`gen.py --max`): port 0.47 s, 45460 KB, original 2.40 s, 69632 KB
