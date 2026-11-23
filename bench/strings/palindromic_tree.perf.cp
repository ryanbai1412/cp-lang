// Tuned: the eertree of palindromic_tree.cpp written out on flat lists, in
// place of `PalindromicTree`, whose `push` also keeps the undo log and front
// letters that `pop` and `push_front` need.
domain Alpha = 0..4
const K = 4
let n = 1_000_000
var s = List.filled(n, 0)
var x = 135792468
for i in 0..n:
    x = (x * 1103515245 + 12345) & 0x7fffffff
    s[i] = (x >> 16) & 3

var next = List.filled(2 * K, -1)
var slink = [0, 0]
var plen = [-1, 0]
var cnt = [0, 0]
var last = 1
unchecked:
    for (pos, c) in s.items:
        var cur = last
        while not (pos - 1 - plen[cur] >= 0 and s[pos - 1 - plen[cur]] == c): cur = slink[cur]
        if next[cur * K + c] == -1:
            let len = plen[cur] + 2
            var link = 1
            if len > 1:
                var p = slink[cur]
                while not (pos - 1 - plen[p] >= 0 and s[pos - 1 - plen[p]] == c): p = slink[p]
                link = next[p * K + c]
            next[cur * K + c] = plen.len
            for _ in 0..K: next.push(-1)
            plen.push(len)
            slink.push(link)
            cnt.push(0)
        last = next[cur * K + c]
        cnt[last] += 1
    for i in (2..cnt.len).rev: cnt[slink[i]] += cnt[i]
var ans = plen.len - 2
for i in 2..cnt.len: ans += cnt[i] * plen[i]
print(ans)
