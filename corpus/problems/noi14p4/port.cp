// NOI 2014 Zoo: for every prefix, count its borders (a proper prefix that is
// also a suffix) that are at most half as long; print the product of
// (count + 1) over all prefixes.
use std.strings.{prefix_function}

type Z = Mod<1_000_000_007>
@!multitest
let s: Str = read()
let pi = prefix_function(s)

var depth: [0..=s.len]Int = 0
var half = 0
var answer: Z = 1
for i in 0..s.len:
    depth[i + 1] = depth[pi[i]] + 1
    if i > 0:
        while half > 0 and s[half] != s[i]: half = pi[half - 1]
        if s[half] == s[i]: half += 1
        while 2 * half > i + 1: half = pi[half - 1]
    answer *= depth[half] + 1
print(answer)
