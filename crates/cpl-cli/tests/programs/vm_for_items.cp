let n: Int = read()
var s: Set<Int> = []
var hs: HashSet<Int> = []
var m: Map<Int, List<Int>> = []
var hm: HashMap<Int, Int> = []
for i in 0..n:
    s.insert(i * 37 % 11)
    hs.insert(i % 5)
    m[i % 3] = [i, i * i]
    hm[i * 7 % 6] = i
var total = 0
var seen: List<Int> = []
for x in s:
    seen.push(x + 100)
    total += x
print(total, seen)
for x in hs: total += x * x
for (k, v) in m: print(k, v)
for (k, v) in m.rev(): print(k, v.len)
for (k, v) in hm: total += k * v
print(total)
for x in s.rev():
    if x > 105: continue
    if x < 3: break
    print("r", x)
let opts: List<?Int> = [Some(3), None, Some(-4)]
let arr: [0..3]?Int = [None, Some(9), Some(0)]
for o in opts: print(o ?? 0)
for o in arr: print(o ?? -1)
