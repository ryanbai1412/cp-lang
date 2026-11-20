// AAC 4 P4 (today/ runnable subset) — original: corpus/submissions/4140376.cpp

let n: 1..=50 = read()
let m: 1..=50 = read()
let s: 1..=50 = read()
let f: 1..=50 = read()
let l: 0..=2500 = read()
let r: 0..=2500 = read()
let arr: [0..m]Int = read()

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
