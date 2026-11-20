// original: 5213639 (noip22p1) -- first draft, written from docs/one-page-cheat-sheet.md only; does not build on proto/cpc
const MOD = 998244353
type Z = Mod<MOD>
let (t, id): (Int, Int) = read()
for tc in 0..t:
    let (n, m, kc, kf): (Int, Int, Int, Int) = read()
    let g: [0..n, 0..m]Char = read()
    var hor: [0..n, 0..=m]Int = 0
    var ver: [0..=n, 0..m]Int = 0
    for i in 0..n:
        for j in (0..m).rev:
            if g[i, j] == '0': hor[i, j] = hor[i, j + 1] + 1
    for j in 0..m:
        for i in (0..n).rev:
            if g[i, j] == '0': ver[i, j] = ver[i + 1, j] + 1
    var ansc: Z = 0
    var ansf: Z = 0
    for j in 0..m:
        var sc: Z = 0
        var sf: Z = 0
        for i in (0..n).rev:
            if g[i, j] == '1':
                sc = 0
                sf = 0
                continue
            let w = max(0, hor[i, j] - 1)
            ansc += sc * w
            ansf += sf * w
            if i + 1 < n:
                let mh = max(0, hor[i + 1, j] - 1)
                sc += mh
                sf += mh * max(0, ver[i + 1, j] - 1)
    print(ansc * kc, ansf * kf)
