// NOI '02 P2: interactive truth-teller / liar / wild-card ring of n = cnt_n + cnt_m + 1
// people. Query every person about both cyclic neighbors via v.cyc(1) / v.cyc(-1),
// find the unique mismatch marking the wild card, and propagate labels around the ring.
// original: corpus/submissions/4793917.cpp

let (cnt_n, cnt_m): (Int, Int) = ask("GetNM")
let n = cnt_n + cnt_m + 1
domain P = 0..n

fn query(i: P, j: P) -> Bool:
    let ans: Int = ask("Ask", i.to_idx + 1, j.to_idx + 1, 0)
    ans == 1

// dir[p, true] = p's answer about p.cyc(1); dir[p, false] = p's answer about p.cyc(-1).
var dir: [P, Bool]Bool = false
for p in P:
    dir[p, true] = query(p, p.cyc(1))
    dir[p, false] = query(p, p.cyc(-1))

let wild = first(p for p in P if dir[p, true] != dir[p.cyc(1), false])!
var ans: [P]Int = 0
ans[wild] = 2

for p in (..wild).rev:
    ans[p] = Int(dir[p, true] != (ans[(p + 1)!] == 0))
for p in wild.r..:
    ans[p] = Int(dir[p, false] != (ans[(p - 1)!] == 0))

for p in P: print("Answer", ans[p])
