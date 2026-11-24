use std.dp.monge_paths
let (n, c): (1..=1e6, 0..=1e9) = read()
let a: [0..n]0..=1000 = read()
var pre = [0]
for x in a: pre.push(pre.last! + x)
let path = monge_paths([i for i in 0..=n], (j, i) => (pre[i] - pre[j]) ** 2 + c)
var (hops, at) = (0, n)
while at > 0:
    hops += 1
    at = path.prev[at]
print(path.val[n], hops)
