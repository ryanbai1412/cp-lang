// Source: An Animal Contest 4 P4 Reindeer Racing, https://dmoj.ca/problem/aac4p4
// Problem: n lanes, m hurdles (n, m <= 2000); the baton starts in lane s, and when its holder hits
//   hurdle j it passes to lane k_j. A banana at (lane i, hurdle j) makes lane i hit hurdle j.
//   Given 0 <= L <= R <= nm, print a count X in L..=R and an n x m 0/1 matrix of X bananas
//   leaving the baton in lane f, or -1 if impossible.
// Solution: greedy on j, the last hurdle with k_j = f. Without one the baton must never move:
//   need s = f and throw only at other lanes. With R = 0 need s = f. If L <= n, hit hurdle j in
//   lane s plus up to L - 1 other lanes. Otherwise hit hurdle j in every lane (whoever holds the
//   baton passes it to f), then fill any cell except lane f after hurdle j until L. O(nm).
// original: corpus/submissions/4140376.cpp

let n: 1..=2000 = read()
let m: 1..=2000 = read()
let s: 1..=n = read()
let f: 1..=n = read()
let l: 0..=n * m = read()
let r: l..=n * m = read()
let arr: [0..m]1..=n = read()

var ans: [0..n, 0..m]Int = 0

fn emit(cnt: Int):
    print(cnt)
    for i in 0..n:
        var row: List<Int> = []
        for j in 0..m:
            row.push(ans[i, j])
        print(row)

fn solve() -> Bool:
    var ind = -1
    for i in 0..m:
        if arr[i] == f:
            ind = i
    if ind == -1:
        if s != f:
            return false
        if l == 0:
            emit(0)
            return true
        var cnt = 0
        for i in 0..n:
            if i != s - 1:
                for j in 0..m:
                    ans[i, j] = 1
                    cnt += 1
                    if cnt == l:
                        emit(l)
                        return true
        return false
    if r == 0:
        if s != f:
            return false
        emit(0)
        return true
    if l <= n:
        ans[s - 1, ind] = 1
        var cnt = 1
        for i in 0..n:
            if i != s - 1 and cnt < l:
                ans[i, ind] = 1
                cnt += 1
        emit(cnt)
        return true
    var cnt = n
    for i in 0..n:
        ans[i, ind] = 1
    for i in 0..n:
        for j in 0..m:
            if not (i == f - 1 and j > ind) and ans[i, j] == 0:
                ans[i, j] = 1
                cnt += 1
                if cnt == l:
                    emit(l)
                    return true
    false

if not solve():
    print(-1)
