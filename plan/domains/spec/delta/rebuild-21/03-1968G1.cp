// CF 1968G1 Division + LCP (easy): binary search on the common prefix length.
@!multitest
let n = read<1..=2e5>()
let l = read<1..=n>()
let r = read<l..=n>()
let s = read<Str>()
let z = z_function(s)

// greedy count of disjoint copies of s[..len] in s; len >= 1
fn pieces(len: Int) -> Int:
    var cnt = 0
    var i = 0
    while i + len <= n:
        if z[i] >= len:
            cnt += 1
            i += len
        else:
            i += 1
    cnt

let k = l
print((0..=n / k).last_true(len => len == 0 or pieces(len) >= k)!)
