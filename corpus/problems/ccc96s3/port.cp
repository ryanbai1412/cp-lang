// Source: CCC 1996 S3 (Pattern Generator), https://dmoj.ca/problem/ccc96s3
// Problem: for each of several pairs n, k (n <= 30, k < 8), print "The bit patterns are" and then
//   all length-n bit patterns with k ones in descending binary order; blank line between cases.
// Solution: backtracking that places a 1 before a 0 at each position, so patterns come out in
//   descending order, and only while the remaining ones fit: O(C(n, k) * n) per case.
// List<Char> prints with no separators, so `print(cur)` writes the pattern.
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
