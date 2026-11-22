// CCO '22 P2 — Rainy Markets (today form for proto/cpc)
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
