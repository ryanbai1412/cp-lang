// expect: ok, output "-17\n-4\n-3\n4"
// rule: judge.9
// tool: run --release {case}
// file: cp.toml "[judge.fast]\nextends = \"generic\"\nisa = \"avx2\"\n"
// judge: fast
// input: "4"
// On an AVX2 judge the max-plus row loop is one branch-free pass; it keeps `none` and the max as the loop does.
let n: 1..=8 = read()
domain S = 0..n
type Row = [S]?Int
fn keep(acc: Row, row: Row, base: Int) -> Row:
    var c = acc
    for l in S:
        if let w = row[l]: c[l] max= Some(base + w)
    c
let row: Row = [S] s => if Int(s) % 3 == 1: None else: Some(Int(s) * 7 - 20)
let acc: Row = [S] s => if Int(s) % 2 == 0: None else: Some(Int(s) - 5)
let out = keep(acc, row, 3)
for s in S: print(out[s] ?? -1)
