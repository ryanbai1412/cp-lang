use std.dp.{dnc_layer}
let (n, k): (1..=2e5, 1..=100) = read()
let a: [0..n]0..=1000 = read()
var pre = [0]
for x in a: pre.push(pre.last! + x)
var layer: List<?Int> = [if i == 0: 0 else: none for i in 0..=n]
for _ in 0..k:
    layer = dnc_layer(layer, (j, i) => (pre[i] - pre[j]) * (pre[i] - pre[j]))
print(layer[n]!)
