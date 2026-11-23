let (n, m): (1..=200000, 0..=200000) = read()
var par: List<Int> = [i for i in 0..=n]  // expect: missing int-vertex-ids, copy_comprehension
fn find(x: Int) -> Int:
    if par[x] == x:
        return x
    par[x] = find(par[x])  // expect: hand_rolled_dsu
    return par[x]
var comps = n
for _ in 0..m:
    let (u, v): (1..=n, 1..=n) = read()
    let a = find(u)
    let b = find(v)
    if a != b:
        par[a] = b
        comps -= 1
print(comps)
