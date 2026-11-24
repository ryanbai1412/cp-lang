// Source: NOI 2014 P4 (Zoo), https://dmoj.ca/problem/noi14p4
// limits: 600 ms, 512 MB
// Problem: for a lowercase string S (|S| <= 1e6), num[i] counts the borders of the prefix S[..i]
//   (proper, both prefix and suffix) of length <= i / 2. Print the product of (num[i] + 1) over
//   all i, mod 1e9 + 7. Up to 5 strings (multitest).
// Solution: with the prefix function pi, the depth of prefix j in the border tree (parent
//   pi[j - 1]) counts its borders, itself included. A second KMP pointer `half` follows the longest border, falling
//   back along pi until 2 half <= i + 1; then num = depth[half]. Amortised O(|S|).

type Z = Mod<1_000_000_007>
@!multitest
let s: Str = read()
let pi = prefix_function(s)

// depth[j]: prefix j's borders, itself included; it hangs under its longest border pi[j - 1] < j
var depth: List<Int> = [0]
for j in 1..=s.len: depth.push(depth[pi[j - 1]] + 1)
var half = 0
var answer: Z = 1
for i in 0..s.len:
    if i > 0:
        while half > 0 and s[half] != s[i]: half = pi[half - 1]
        if s[half] == s[i]: half += 1
        while 2 * half > i + 1: half = pi[half - 1]
    answer *= depth[half] + 1
print(answer)
