# cco16p6

Score: 3/5
Dream: 4/5

The header derives the price rule and the bucket trick, and the program follows it with std `SegTree` and `right_while`. It is the C++'s bucket encoding kept, with a special "rich" last bucket whose sum is not `index * count`; the friction reports that and names the structure it wants.

## Checklist failures

- idiomatic-code/R6: buckets cross as raw `Int` (`fn put(b: Int, ...)`, `fn add(b: Int, ...)`, `cut`, `top`, `base`) and are wrapped by `Bucket(b)` at every call.
- idiomatic-code/R2: the bucket encoding and the special `rich` slot are the C++ construction; the editorial's object is a multiset of prices with a global shift.

## Owed rewrites

- Type `put`/`add` and `top` by `Bucket` and convert once where the price arithmetic needs an `Int`.

## Language gaps

- The rich pirate breaks the bucket encoding; an `OffsetMultiset` would remove it: research/frictions/cco16p6.md.

## Dream notes

- `OffsetMultiset` with `smallest_sum`, `shift` and `add` names the object. `prices.keep_smallest(v - 1).raise_to(1)` is unclear about the unbought voters (they should come back at price 1, `i - need` of them, as the port's `add(base, i - need)` says). No header.
