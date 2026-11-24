# coci11c4p4 frictions

## Dream

```cp
let fence = I.sorted(by: rank).zip(boards.sorted()).to_table()
```

## Dealing sorted boards onto ranked positions

There is no table built from (key, value) pairs, so the deal is a loop writing `fence[i] = b`
over `zip`, and a domain has no `sorted`, so its positions are collected first.

## Fresh runs

`cpl run` on a release build of main cae8efb22.

```cp
domain I = 1..=3
let boards = [5, 7, 6]
let fence = zip(I.sorted(by: i => -i.ord), boards.sorted()).to_table()
print(fence[I.first!])
```

```text
error[E0203]: a `List<(I, Int)>` has no method `to_table`
  --> probe.cp:3:61
3 | let fence = zip(I.sorted(by: i => -i.ord), boards.sorted()).to_table()
  |                                                             ^^^^^^^^
```

**Status:** open for the table; fixed for the domain: `I.sorted(by:)` checks without `.collect()`.
