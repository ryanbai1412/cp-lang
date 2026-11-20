// NOI 2014 Zoo: for every prefix, count its borders (a proper prefix that is
// also a suffix) that are at most half as long; print the product of
// (count + 1) over all prefixes.
type Z = Mod<1_000_000_007>
@!multitest
let s: Str = read()

// border[p]: the length of the longest border of the prefix of length p;
// depth[p]: how many borders it has, counting itself.
var border: [0..=s.len]Int = 0
var depth: [0..=s.len]Int = 1
depth[0] = 0

// b is a border of s[..i]. The longest border of s[..=i]: b or one of b's
// borders, grown by s[i].
fn grow(b: Int, i: Int) -> Int:
    var b = b
    while b > 0 and s[b] != s[i]: b = border[b]
    if s[b] == s[i]: b + 1 else: b

var half = 0                                // longest border of the prefix that fits in half of it
var answer: Z = 1
for i in 1..s.len:
    border[i + 1] = grow(border[i], i)
    depth[i + 1] = depth[border[i + 1]] + 1
    half = grow(half, i)
    while 2 * half > i + 1: half = border[half]
    answer *= depth[half] + 1
print(answer)
