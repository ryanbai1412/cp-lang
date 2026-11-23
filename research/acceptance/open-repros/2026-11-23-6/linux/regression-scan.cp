let a = [1, 2, 3, 2, 1]
let sc = a.scan(+)
print(sc)
let i: Int = 2
print(sc[i], sc[a.len - 1])
for s in 0..=5:
    print(s, sc[s])
