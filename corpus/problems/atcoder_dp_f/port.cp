// Source: AtCoder Educational DP Contest F (LCS), https://atcoder.jp/contests/dp/tasks/dp_f
// limits: 2000 ms, 1024 MB
// Problem: two lowercase strings s, t of length 1..=3000. Print one longest common
//   subsequence (any one is accepted; the empty line when there is none).
// Solution: common[i, j] is the LCS length of the suffixes from i and j. Equal heads are
//   always worth taking (any LCS can be changed to use them), so common[i, j] is
//   1 + common[i+1, j+1] then, else the better of dropping either head. Walk from the start:
//   take equal heads, otherwise drop a head whose loss keeps the length. O(|s| |t|).
// original: corpus/submissions/4131645.cpp
let a: Str<1..=3000> = read()
let b: Str<1..=3000> = read()
domain I = 0..=a.len
domain J = 0..=b.len

var common: [I, J]Int = 0
for i in I.rev:
    for j in J.rev:
        if let i1 = i.next and let j1 = j.next:
            common[i, j] = if a[i.label] == b[j.label]: 1 + common[i1, j1] else: max(common[i1, j], common[i, j1])

var out: List<Char> = []
var i = I.first!
var j = J.first!
while common[i, j] > 0:
    if a[i.label] == b[j.label]:
        out.push(a[i.label])
        i = i.next!
        j = j.next!
    elif common[i.next!, j] == common[i, j]: i = i.next!
    else: j = j.next!
print(Str(out))
