# ceoi19pp2 frictions

## Dream

```cp
var kept: List<Int> = []
for (b, high) in bs.decoded(...).with_prefix_max():
```

## Prefix maximum as an optional

The prefix maximum starts empty, so the port keeps `var high: ?Int` and tests
`high == None or x > high!`; the value it decodes depends on the previous answer, so the prefix
maximum cannot be a `scan(max)` over the input.
