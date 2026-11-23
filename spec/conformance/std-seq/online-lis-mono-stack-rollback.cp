// expect: ok, output "2 2 YES\n1 2\n1 NO\n0\n0 YES 2\n1 YES\n2 YES\nYES\n0 0 2\n1 2\n3 0 YES\nYES"
use std.seq.{OnlineLis, MonoStack}
var t = OnlineLis<Int>()
t.push(5)
t.push(1)
print(t.push(7), t.len, yes(OnlineLis<Int>().is_empty))
var r = OnlineLis<Int>()
for x in [3, 1, 2]: r.push(x)
print(r.reconstruct())
var u = OnlineLis<Int>()
u.push(2)
let m = u.mark()
u.push(5)
u.undo_to(m)
print(u.len, yes(u.is_empty))
u.clear()
print(u.len)
var s = MonoStack<Int, Int>()
s.push(0, 5)
print(s.push(1, 3) ?? -1, yes(s.top == Some((index: 1, val: 3))), s.len)
let sm = s.mark()
s.push(2, 9)
print(s.len, yes(s.top == Some((index: 2, val: 9))))
s.undo_to(sm)
print(s.len, yes(s.top == Some((index: 1, val: 3))))
s.clear()
print(yes(s.is_empty))
var w = MonoStack<Int, Int>(keep_smaller: false)
w.push(0, 1)
print(w.push(1, 4) ?? -1, w.push(2, 2) ?? -1, w.push(3, 3) ?? -1)
print([1, 2, 3].chunks_padded(2, 0).collect(), yes((u: 1, v: 2).swap == (u: 2, v: 1)))
var ok = true
var seed = 7
for _ in 0..40:
    var a: List<Int> = []
    var lis = OnlineLis<Int>()
    var st = MonoStack<Int, Int>()
    var marks: List<(Int, Int, Int)> = []
    for step in 0..30:
        seed = (seed * 1103515245 + 12345) % 2147483648
        if seed % 5 == 0 and marks.len > 0:
            let (n, ml, ms) = marks.pop()!
            while a.len > n: a.pop()
            lis.undo_to(ml)
            st.undo_to(ms)
        else:
            if seed % 3 == 0: marks.push((a.len, lis.mark(), st.mark()))
            let x = seed / 7 % 10
            let got = lis.push(x)
            var best = 1
            var dp: List<Int> = []
            a.push(x)
            for i in 0..a.len:
                var d = 1
                for j in 0..i:
                    if a[j] < a[i] and dp[j] + 1 > d: d = dp[j] + 1
                dp.push(d)
            if got != dp[a.len - 1]: ok = false
            var near: ?Int = None
            for j in 0..a.len - 1:
                if a[j] > x: near = Some(j)
            if st.push(a.len - 1, x) != near: ok = false
        var best = 0
        var dp2: List<Int> = []
        for i in 0..a.len:
            var d = 1
            for j in 0..i:
                if a[j] < a[i] and dp2[j] + 1 > d: d = dp2[j] + 1
            dp2.push(d)
            best max= d
        if lis.len != best: ok = false
        let rec = lis.reconstruct()
        if rec.len != best: ok = false
        for k in 1..rec.len:
            if not (rec[k - 1] < rec[k] and a[rec[k - 1]] < a[rec[k]]): ok = false
print(yes(ok))
