// Source: IOI 2000 P1 (Palindrome), https://dmoj.ca/problem/ioi00p1
// Problem: given a string of length 3 <= N <= 5000 (letters, case-sensitive, and digits),
//   print the fewest characters to insert anywhere to make it a palindrome. Memory 32 MB.
// Solution: interval DP: need(i, j) for s[i..=j] is need(i + 1, j - 1) when s[i] == s[j], else
//   1 + min(need(i + 1, j), need(i, j - 1)). Run i downward keeping only the rows for i and
//   i + 1 (inner), so it is O(N^2) time and O(N) memory.
// original: corpus/submissions/4682427.cpp
let n: 3..=5000 = read()
domain I = 0..n
fn word() -> [I]Char: [I] _ => 'a' + rand(0, 2)
let s: [I]Char = read(gen: word)

var inner: [I]Int = 0  // the row for i + 1
for i in I.rev:
    var need: [I]Int = 0
    for j in i.after:
        need[j] = if s[i] == s[j]: inner[j.prev!] else: 1 + min(inner[j], need[j.prev!])
    inner = need
print(inner[I.last!])
