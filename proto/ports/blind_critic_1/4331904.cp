// Corpus submission 4331904: minimum-token string segmentation with lexicographic reconstruction
let n: 1..=100 = read()
let m: 1..=1000 = read()
let s: [domain S]Char = read()
domain D = 1..=n

var chars: [D]Char = 'a'
var words: [D]List<Char> = []
for d in D:
    let c: Char = read()
    let w: [domain W]Char = read()
    chars[d] = c
    for k in W:
        words[d].push(w[k])

fn matches_at(pos: Int, d: D) -> Bool:
    let wlen = words[d].len
    if pos + wlen > s.len:
        return false
    for k in 0..wlen:
        if s[S.nth(pos + k)] != words[d][k]:
            return false
    true

var dp: [S.bounds]?Int = none
dp[S.bounds.last] = 0

for i in S.rev:
    for d in D:
        if matches_at(i.idx, d):
            if let nxt = dp[i.r(words[d].len)]:
                dp[i.l] min= 1 + nxt

var ans: List<Char> = []
var pos = 0
while pos < s.len:
    let cur_dp = dp[S.nth(pos).l]!
    var best_c: ?Char = none
    var best_nxt = 0
    for d in D:
        let wlen = words[d].len
        if matches_at(pos, d):
            if let nxt = dp[S.nth(pos).r(wlen)]:
                if cur_dp == 1 + nxt:
                    let c = chars[d]
                    let nxt_pos = pos + wlen
                    if best_c == none or c < best_c! or (c == best_c! and nxt_pos < best_nxt):
                        best_c = c
                        best_nxt = nxt_pos
    ans.push(best_c!)
    pos = best_nxt

print(ans)
