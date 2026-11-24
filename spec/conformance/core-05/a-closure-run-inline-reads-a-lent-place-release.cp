// expect: ok
// tool: run --release
// has: "4 3"
// file: cp.toml "[libs]\nkit = \"kit\"\n"
// file: kit/m.cp "pub fn pick(n: Int, ok: Int -> Bool) -> Int:\n    var c = 0\n    for k in 0..n:\n        if ok(k): c += 1\n    c\n"
// A closure given to a library fn that runs inline at its call reads `r`, an element of `rows` lent in place, so both engines read the same element and nothing copies it.
use kit.m.{pick}
let rows: List<List<Int>> = [[1, 2], [3, 4, 5, 6, 7]]
let r = rows[1]
print(pick(r.len, k => r[k] > 3), pick(rows[0].len, k => rows[0][k] > 0) + 1)
