domain R = 0..3
domain C = 2..=4
var g: [R, C]Int = 0
for p in g.keys:
    g[p] = p.0.label * 10 + p.1.label
    if p.1.label == 3: continue
    let (r, c) = p
    print(r.label, c.label, g[p])
for p in g.keys:
    if p.0.label == 2: break
    print(p)
