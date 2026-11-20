// CEOI 2022 P2, Homework — ideal CP Lang sketch
// Category: Unusual I/O (recursive grammar string `min(...,...)`/`max(...,...)`/`?`) + grammar `gen` + `spec`
// Original C++: corpus/submissions/4727898.cpp (116 LOC raw, ~72 LOC stripped)
// Buildable today-form: proto/ports/scout_batch_19/today/ceoi22p2.cp (verified 40 trees up to 1000 leaves)

let s: Str = read()

// Stack-safe O(|s|) interval DP over expression string without recursion depth overflow
// Each subtree tracks (min_lt, min_gt): minimum count of leaves forced strictly < and > the result.
fn count_possible(expr: Str) -> Int:
    var ops: List<Char> = []        // 'n' = min, 'x' = max
    var vals: List<(Int, Int)> = []
    var leaves = 0
    var i = 0
    while i < expr.len:
        match expr[i]:
            '?' =>
                leaves += 1
                vals.push((0, 0))
                i += 1
            'm' =>
                ops.push(if expr[i + 1] == 'i': 'n' else: 'x')
                i += 4 // skip "min(" or "max("
            ')' =>
                let op = ops.pop()!
                let (r_lt, r_gt) = vals.pop()!
                let (l_lt, l_gt) = vals.pop()!
                if op == 'n':
                    vals.push((min(l_lt, r_lt), l_gt + r_gt + 1))
                else:
                    vals.push((l_lt + r_lt + 1, min(l_gt, r_gt)))
                i += 1
            _ => i += 1
    let (lt, gt) = vals[0]
    leaves - lt - gt

print(count_possible(s))

// WISH B19-3: constructive grammar `gen` (rejection-sampling random strings has 0% hit rate;
// shrinking shrinks `leaves` directly and re-samples subtrees while staying grammatically valid!)
gen s:
    fn build(leaves: Int) -> Str:
        if leaves == 1: return "?"
        let left = rng.int(1..<leaves)
        let op = rng.choice(["min", "max"])
        f"{op}({build(left)},{build(leaves - left)})"
    let leaves = rng.int(1..=min(size, 1_000_000))
    yield build(leaves)

// Co-located brute-force `spec` for small trees (leaves <= 8): enumerate all permutations of 1..=N!
spec count_possible(expr: Str) -> Int where count(ch == '?' for ch in expr) <= 8:
    fn eval(mut cur: Scanner, vals: List<Int>, mut next_leaf: Int) -> Int:
        if cur.eat("?"):
            let v = vals[next_leaf]
            next_leaf += 1
            return v
        let is_min = cur.eat("min(")
        if not is_min: assert cur.eat("max(")
        let a = eval(mut cur, vals, mut next_leaf)
        assert cur.eat(",")
        let b = eval(mut cur, vals, mut next_leaf)
        assert cur.eat(")")
        if is_min: min(a, b) else: max(a, b)

    let n = count(ch == '?' for ch in expr)
    var seen: Set<Int> = {}
    for p in permutations(1..=n):
        var sc = Scanner(expr)
        var k = 0
        seen.insert(eval(mut sc, p, mut k))
    seen.len
