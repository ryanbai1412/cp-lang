// cp: 0.1 8ef64b9+dirty
let n: 1..=1e5 = read()
domain Bar = 1..=n
type Kids = (l: ?Bar, r: ?Bar)

fn is_forest(k: [Bar]Kids) -> Bool:
    var parents: [Bar]Int = 0
    for v in Bar:
        if let c = k[v].l:
            if c <= v:
                return false
            parents[c] += 1
        if let c = k[v].r:
            if c <= v:
                return false
            parents[c] += 1
    all(parents[v] <= 1 for v in Bar)

// random binary tree rooted at Bar.first, for stress tests
fn random_mobile() -> [Bar]Kids:
    var kids: [Bar]Kids = [Bar] _ => (l: None, r: None)
    var free: List<(Bar, Bool)> = [(Bar.first, true), (Bar.first, false)]
    for v in Bar:
        if v != Bar.first:
            let i = rand(0, free.len - 1)
            let (u, left) = free[i]
            free[i] = free.last!
            free.pop()
            if left: kids[u].l = v else: kids[u].r = v
            free.push((v, true))
            free.push((v, false))
    kids

let kids: [Bar]Kids = read(gen: random_mobile) where is_forest(kids)

var swaps = 0

// toy depths under u as (deepest, shallowest), or none if no swaps can sort them
fn span(u: ?Bar, d: Int) -> ?(Int, Int):
    let v = u else:
        return (d, d)
    let (ld, ls) = span(kids[v].l, d + 1)?
    let (rd, rs) = span(kids[v].r, d + 1)?
    if ls >= rd:
        return (ld, rs)
    if rs >= ld:
        swaps += 1
        return (rd, ls)
    return None

fn answer() -> ?Int:
    let (deep, shallow) = span(Bar.first, 0)?
    if deep - shallow > 1:
        return None
    return swaps

print(answer() ?? -1)
