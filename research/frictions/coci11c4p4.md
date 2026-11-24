# coci11c4p4 frictions

## Dream

```cp
let fence = I.sorted(by: rank).zip(boards.sorted()).to_table()
```

## Dealing sorted boards onto ranked positions

There is no table built from (key, value) pairs, so the deal is a loop writing `fence[i] = b`
over `zip`, and a domain has no `sorted`, so its positions are collected first.
