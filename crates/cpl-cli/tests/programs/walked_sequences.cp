// A range, a domain and `Bool` are walked where a sequence is wanted.
let n = 5
let w = [3, 1, 4, 1, 5]
let order = (0..n).sorted(by: i => w[i])
print(order)
domain I = 0..n
let by_dom = I.sorted(by: i => -w[i.to_idx])
print([i.ord for i in by_dom])
for b in Bool: print(yes(b))
print(count(b for b in Bool if b), [x * 2 for x in 0..3], sum(0..4), (1..=3).max()!)
