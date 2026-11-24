let n: 2..=200000 = read()
domain V = 1..=n
let par: [0..n - 1]V = read()
let t = Rooted.from_parents([V] v => if v == 1: None else: Some(par[v.ord - 1]))
let (a, b): (V, V) = read()
print(if t.depth(a) >= t.depth(b): a else: b)
