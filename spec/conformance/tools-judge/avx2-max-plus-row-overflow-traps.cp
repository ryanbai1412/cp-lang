// expect: exit 3
// rule: judge.9
// tool: run --release --checks all {case}
// file: cp.toml "[judge.fast]\nextends = \"generic\"\nisa = \"avx2\"\n"
// judge: fast
// input: "4 9223372036854775000"
// has: "`9223372036854775000 + 1000` does not fit in an `Int`"
// The branch-free pass folds each add's overflow into one flag, and an overflow traps as the loop does.
let n: 1..=8 = read()
domain S = 0..n
type Row = [S]?Int
fn keep(acc: Row, row: Row, base: Int) -> Row:
    var c = acc.clone()
    for l in S:
        if let w = row[l]: c[l] max= Some(base + w)
    c
let base: Int = read()
let row: Row = [S] s => Some(Int(s) * 1000)
let out = keep(row, row, base)
print(out[S.first!] ?? -1)
