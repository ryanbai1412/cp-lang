// A range, a domain and `Bool` are walked where a sequence is wanted.
let n = 5
let w = [3, 1, 4, 1, 5]
let order = sorted(0..n, by: i => w[i])
print(order)
domain I = 0..n
let by_dom = sorted(I, by: i => -w[i.to_idx])
print(by_dom)
for b in Bool: print(b)
print(count(b for b in Bool), [x * 2 for x in 0..3], sum(0..4), max(1..=3))
