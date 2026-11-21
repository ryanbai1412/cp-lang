var m: Map<Int, Int> = [3: 30, 1: 10, 2: 20]
var hm: HashMap<Int, Int> = [10: 1, 20: 2, 30: 3]

var ks: List<Int> = []
for k in m.keys:
    ks.push(k)
print(ks)

var vs: List<Int> = []
for v in m.values:
    vs.push(v)
print(vs)

var rks: List<Int> = []
for k in m.keys.rev:
    rks.push(k)
print(rks)

print(m.keys.sum(), m.values.sum(), m.values.max() ?? 0, m.values.min() ?? 0)
print(sum(m.values), m.values.count(v => v >= 20), m.keys.any(k => k == 2), m.keys.all(k => k > 0))
print([v * 2 for v in m.values])
print(m.keys, m.values)

print(hm.keys.sum(), hm.values.sum(), sum(hm.values), hm.values.max() ?? 0)
var hsum = 0
for v in hm.values:
    hsum += v
print(hsum)
