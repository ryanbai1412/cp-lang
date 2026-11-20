// Codeforces 2262 D: a token sits on some cell; op 1 increments a[cur], op 2
// jumps to cell a[cur].  Turn a into b or report -1.  Only cells whose target
// chain b[i] -> b[b[i]] -> .. reaches the last differing cell may change.
// Scanning right to left, `via[i]` records where the token must travel next
// from i (i itself: go back down first; none: just finish this cell).
@!multitest
let n: 1..=2000 = read()
let a_in: [0..n]1..=n = read()
let b_in: [0..n]1..=n = read()
var a = [x - 1 for x in a_in]
let b = [x - 1 for x in b_in]

fn plan() -> ?(start: Int, ops: List<Int>):
    if any(a[i] > b[i] for i in 0..n): return none
    var last = n - 1
    while last > 0 and a[last] == b[last]: last -= 1
    var bound = last
    var okay: [0..n]Bool = false
    okay[last] = true
    var via: [0..n]?Int = none
    var low = last
    for i in (0..last).rev:
        okay[i] = b[i] > i and b[i] <= bound and okay[b[i]]
        if a[i] == b[i]: continue
        if not okay[i]: return none
        if i < a[low]:
            bound = i
            via[low] = low
            low = i
        elif a[i] < a[low]:
            via[low] = i
            low = i
    via[low] = low

    var ops: List<Int> = []
    var cur = low
    while a[last] != b[last]:
        var jump = false
        var decided = false
        if via[cur] != none and via[cur] != cur:
            let t = via[cur]!
            if via[t] == none: via[cur] = cur
            else:
                jump = a[cur] == t
                decided = true
        if not decided and via[cur] == cur:
            if a[cur] < cur:
                jump = a[a[cur]] != b[a[cur]]
                decided = true
            else: via[cur] = none
        if not decided: jump = a[cur] == b[cur]
        if jump:
            ops.push(2)
            cur = a[cur]
        else:
            ops.push(1)
            a[cur] += 1
    (low, ops)

let res = plan()
if res == none:
    print(-1)
    return
print(res!.ops.len, res!.start + 1)
print(res!.ops)
