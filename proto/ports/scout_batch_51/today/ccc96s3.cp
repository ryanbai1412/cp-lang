// Runnable today/ port of CCC '96 S3 (Pattern Generator) for proto/cpc.
// Uses List<Char> (which prints with zero separators) and recursive backtracking
// matching the v2.1 block-scoped defer solution.
// original: corpus/submissions/5515585.cpp

fn main_solve():
    let t: Int = read()
    for tc in 0..t:
        if tc > 0: print("")
        let n: Int = read()
        let k: Int = read()
        print("The bit patterns are")
        var cur: List<Char> = []
        fn dfs(rem: Int, ones: Int):
            if rem == 0:
                print(cur)
                return
            if ones > 0:
                cur.push('1')
                dfs(rem - 1, ones - 1)
                let _ = cur.pop()
            if rem > ones:
                cur.push('0')
                dfs(rem - 1, ones)
                let _ = cur.pop()
        dfs(n, k)

main_solve()
