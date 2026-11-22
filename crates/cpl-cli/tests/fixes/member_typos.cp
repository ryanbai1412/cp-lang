domain V = 0..4
let xs = List.fille(4, 2)
let u = xs.distinc()
let f = V.firs(v => v.ord > 0)
let ok = u.len.is_po2
print(f, ok)
