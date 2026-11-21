use std.strings.{z_function, prefix_function}

let n = 1_000_000
var s = List.filled(n, 0)
var x = 123456789
for i in 0..n:
    x = (x * 1103515245 + 12345) & 0x7fffffff
    s[i] = (x >> 16) % 3
let z = z_function(s)
let p = prefix_function(s)
var ans = 0
for i in 0..n:
    ans += z[i] * 3 + p[i]
print(ans)
