// CCC '96 S3 (Pattern Generator): print all length-n binary strings with k ones in
// descending lexicographical order, with blank lines between test cases.
// Uses block-scoped `defer` (v2.1 U6) inside each branch of recursive backtracking.
// original: corpus/submissions/5515585.cpp

let t: 1..=100 = read()
for tc in 0..t:
    if tc > 0: print("")
    let (n, k): (1..=30, 0..=n) = read()
    print("The bit patterns are")
    var cur: List<Char> = []
    fn dfs(rem: Int, ones: Int):
        if rem == 0:
            print(cur)
            return
        if ones > 0:
            cur.push('1')
            defer cur.pop()
            dfs(rem - 1, ones - 1)
        if rem > ones:
            cur.push('0')
            defer cur.pop()
            dfs(rem - 1, ones)
    dfs(n, k)
