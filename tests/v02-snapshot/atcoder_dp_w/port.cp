// Source: AtCoder Educational DP Contest W (Intervals), https://atcoder.jp/contests/dp/tasks/dp_w
// Problem: a 0/1 string of length n <= 2e5 and m <= 2e5 rules (l, r, a), |a| <= 1e9: a rule
//   adds a to the score if the string has a 1 in l..=r. Print the largest possible score.
// Solution: dp[i] = the best score with the last 1 at i (0: none), counting the rules with
//   r <= i. A SegTree with range add and max over i: set dp[i] = max dp[0..i), then each rule
//   ending at i adds a to dp[l..=i], the states whose last 1 lies inside it. The answer is the
//   max over all states: O((n + m) log n).
// original: corpus/submissions/4133460.cpp

let (n, m): (Int, Int) = read()
domain Pos = 0..=n
domain Rule = 0..m
let rules: [Rule](l: Int, r: Int, a: Int) = read()

var by_r: [Pos]List<(l: Pos, a: Int)> = [Pos] _ => []
for k in Rule:
    by_r[Pos(rules[k].r)].push((l: Pos(rules[k].l), a: rules[k].a))

var st = SegTree<Pos, Int, Int>(max, fill: 0, act: add)
st.set(Pos(0), 0)
for i in Pos(1)..=Pos(n):
    let best_prev = st.fold(Pos(0)..i)
    st.set(i, best_prev)
    for seg in by_r[i]:
        st.apply(seg.l..=i, seg.a)
print(st.fold(Pos(0)..=Pos(n)))
