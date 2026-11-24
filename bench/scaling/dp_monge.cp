use std.dp.{monotone_minima, dnc_layer, monge_path, monge_paths, smawk}
let n: Int = read()
var s = 0
let xs = [i * 7919 % n for i in 0..n]

let sq_smawk: (Int, Int) -> Int = (a, b) => (b - a) * (b - a) + 7 // within: dp.smawk
let sq_path: (Int, Int) -> Int = (a, b) => (b - a) * (b - a) + 7 // within: dp.monge_path
let sq_paths: (Int, Int) -> Int = (a, b) => (b - a) * (b - a) + 7 // within: dp.monge_paths
let sq_dnc: (Int, Int) -> Int = (a, b) => (b - a) * (b - a) + 7 // within: dp.dnc_layer
let sqo: (Int, Int) -> ?Int = (i, j) => Some((j - i) * (j - i)) // within: dp.monotone_minima
for _ in 0..8:
    s += monotone_minima(n, n, sqo).len // timed: dp.monotone_minima
    s += smawk(n, n, sq_smawk).len // timed: dp.smawk
    s += monge_path(xs.sorted(), sq_path).val // timed: dp.monge_path
    s += monge_paths(xs.sorted(), sq_paths).val.len // timed: dp.monge_paths
let prev = [if i == 0: Some(0) else: None for i in 0..n]
for _ in 0..8:
    s += dnc_layer(prev, sq_dnc).len // timed: dp.dnc_layer
print(s)
