// Source: CCO 2022 P2 (Rainy Markets), https://dmoj.ca/problem/cco22p2
// Problem: n <= 1e6 bus stops hold B_i <= 2e9 people; market i, between stops i and i + 1, has
//   P_i <= 1e9 people and U_i <= 1e9 umbrellas at cost 1. Each person goes to stop i, to stop
//   i + 1 or buys an umbrella. Print NO if someone gets wet, else YES, the least money spent and
//   per market how many go left, buy, go right.
// Solution: greedy left to right: market i fills what is left of stop i, then stop i + 1, and the
//   rest buy umbrellas. If they run out, earlier markets with spare umbrellas buy instead and
//   each market between shifts that many people from its right stop to its left, freeing room
//   in stop i. A stack keeps the spare umbrellas, nearest on top, truncated to the right-goers z
//   since every shifted unit passes through them; shifts go into a difference array over stops
//   applied at the end. O(n) amortized.
// original: corpus/submissions/4801882.cpp
let n: 2..=1_000_000 = read()
domain Bus = 0..n
domain Mkt = 0..n - 1
let bus: [Bus]Int = read()
let ppl: [Mkt]Int = read()
let umbr: [Mkt]Int = read()

var st_at: List<Int> = []
var st_cnt: List<Int> = []
var st_pref: List<Int> = []
var total = 0

fn truncate(cap: Int):
    if total <= cap: return
    while st_at.len > 0 and st_pref[st_at.len - 1] >= cap:
        st_at.pop()
        st_cnt.pop()
        st_pref.pop()
    if st_at.len > 0:
        let top = st_at.len - 1
        st_cnt[top] = cap - st_pref[top]
    total = cap

var left_bus: [Mkt]Int = 0
var rain: [Mkt]Int = 0
var right_bus: [Mkt]Int = 0
var diff: [Bus]Int = 0
var carry = 0
var ok = true

for i in 0..n - 1:
    if ok:
        let mi = Mkt(i)
        var x = min(ppl[mi], bus[Bus(i)] - carry)
        let z = min(bus[Bus(i + 1)], ppl[mi] - x)
        var y = ppl[mi] - x - z
        if y <= umbr[mi]:
            let rem = umbr[mi] - y
            st_at.push(i)
            st_cnt.push(rem)
            st_pref.push(total)
            total += rem
        else:
            let orig_need = y - umbr[mi]
            if total < orig_need:
                ok = false
            else:
                var need = orig_need
                total -= need
                while need > 0:
                    let top = st_at.len - 1
                    let j = st_at[top]
                    let take = min(need, st_cnt[top])
                    need -= take
                    rain[Mkt(j)] += take
                    right_bus[Mkt(j)] -= take
                    diff[Bus(j + 1)] += take
                    diff[Bus(i)] -= take
                    st_cnt[top] -= take
                    if st_cnt[top] == 0:
                        st_at.pop()
                        st_cnt.pop()
                        st_pref.pop()
                x += orig_need
                y -= orig_need
        if ok:
            left_bus[mi] = x
            rain[mi] = y
            right_bus[mi] = z
            carry = z
            truncate(z)

if not ok:
    print("NO")
else:
    var shift = 0
    var used = 0
    for i in 0..n - 1:
        let mi = Mkt(i)
        shift += diff[Bus(i)]
        left_bus[mi] += shift
        right_bus[mi] -= shift
        used += rain[mi]
    print("YES")
    print(used)
    for i in Mkt:
        print(left_bus[i], rain[i], right_bus[i])
