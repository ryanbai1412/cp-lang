// Not a judge-accepted submission: written for the corpus and checked against the sample and an exhaustive brute.
// Reads the sample grader's input (N Q, then H, then Q lines "L R") and prints minimum_costs.
// With m a highest mountain in [l, r], the meeting is at m, right of it (everyone left of m pays H[m],
// plus f(m + 1, r)) or, mirrored, left of it. f(L, x) for x across a Cartesian-tree node's range
// comes from its children's: at x the right option exceeds the left one by a nondecreasing amount,
// so a binary search splits [m, R] into a linear piece and the right child's values plus a constant,
// kept in a segment tree; each query reads f(m + 1, r) when node m is merged.
#include <bits/stdc++.h>
using namespace std;
using ll = long long;

struct Seg {
    int n;
    vector<ll> a, b, add;
    vector<char> set;
    Seg(int n) : n(n), a(4 * n), b(4 * n), add(4 * n), set(4 * n) {}
    void applySet(int o, ll A, ll B) { set[o] = 1, a[o] = A, b[o] = B, add[o] = 0; }
    void applyAdd(int o, ll c) {
        if (set[o]) b[o] += c;
        else add[o] += c;
    }
    void push(int o) {
        for (int c : {2 * o, 2 * o + 1}) {
            if (set[o]) applySet(c, a[o], b[o]);
            if (add[o]) applyAdd(c, add[o]);
        }
        set[o] = 0, add[o] = 0;
    }
    void assign(int o, int l, int r, int ql, int qr, ll A, ll B) {
        if (qr < l || r < ql) return;
        if (ql <= l && r <= qr) return applySet(o, A, B);
        push(o);
        int m = (l + r) / 2;
        assign(2 * o, l, m, ql, qr, A, B), assign(2 * o + 1, m + 1, r, ql, qr, A, B);
    }
    void plus(int o, int l, int r, int ql, int qr, ll c) {
        if (qr < l || r < ql) return;
        if (ql <= l && r <= qr) return applyAdd(o, c);
        push(o);
        int m = (l + r) / 2;
        plus(2 * o, l, m, ql, qr, c), plus(2 * o + 1, m + 1, r, ql, qr, c);
    }
    ll get(int x) {
        int o = 1, l = 0, r = n - 1;
        while (l < r) {
            if (set[o]) return a[o] * x + b[o];
            push(o);
            int m = (l + r) / 2;
            if (x <= m) o = 2 * o, r = m;
            else o = 2 * o + 1, l = m + 1;
        }
        return (set[o] ? a[o] * x + b[o] : 0) + add[o];
    }
};

// For each query, the cost when meeting at its highest (leftmost-highest) mountain m or right of it.
vector<ll> solve(const vector<ll>& h, const vector<pair<int, int>>& qs) {
    int n = h.size(), lg = __lg(n) + 1;
    vector<vector<int>> sp(lg, vector<int>(n));
    iota(sp[0].begin(), sp[0].end(), 0);
    auto better = [&](int i, int j) { return h[j] > h[i] || (h[j] == h[i] && j < i) ? j : i; };
    for (int k = 1; k < lg; k++)
        for (int i = 0; i + (1 << k) <= n; i++) sp[k][i] = better(sp[k - 1][i], sp[k - 1][i + (1 << (k - 1))]);
    auto top = [&](int l, int r) { int k = __lg(r - l + 1); return better(sp[k][l], sp[k][r - (1 << k) + 1]); };
    vector<int> lc(n, -1), rc(n, -1), st;
    for (int i = 0; i < n; i++) {
        int last = -1;
        while (!st.empty() && h[st.back()] < h[i]) last = st.back(), st.pop_back();
        lc[i] = last;
        if (!st.empty()) rc[st.back()] = i;
        st.push_back(i);
    }
    vector<ll> res(qs.size());
    vector<vector<int>> at(n);
    for (int i = 0; i < (int)qs.size(); i++) {
        auto [l, r] = qs[i];
        int m = top(l, r);
        res[i] = h[m] * (r - l + 1);
        if (m < r) at[m].push_back(i);
    }
    Seg seg(n);
    vector<int> L(n), R(n), order;
    vector<pair<int, int>> stk = {{st[0], 0}};
    while (!stk.empty()) {
        auto [v, s] = stk.back();
        stk.pop_back();
        if (s == 0) {
            stk.push_back({v, 1});
            if (rc[v] >= 0) stk.push_back({rc[v], 0});
            if (lc[v] >= 0) stk.push_back({lc[v], 0});
            continue;
        }
        L[v] = lc[v] >= 0 ? L[lc[v]] : v;
        R[v] = rc[v] >= 0 ? R[rc[v]] : v;
        for (int i : at[v]) {
            auto [l, r] = qs[i];
            res[i] = min(res[i], h[v] * (v - l + 1) + seg.get(r));
        }
        ll gl = lc[v] >= 0 ? seg.get(v - 1) : 0;
        ll lpay = h[v] * (v - L[v] + 1);
        // left option at x: gl + h[v] * (x - v + 1); right option: lpay + g(x) for x > v
        int lo = v, hi = R[v];
        while (lo < hi) {
            int x = (lo + hi + 1) / 2;
            if (gl + h[v] * (x - v + 1) <= lpay + seg.get(x)) lo = x;
            else hi = x - 1;
        }
        if (lo < R[v]) seg.plus(1, 0, n - 1, lo + 1, R[v], lpay);
        seg.assign(1, 0, n - 1, v, lo, h[v], gl + h[v] * (1 - v));
    }
    return res;
}

int main() {
    cin.tie(nullptr)->sync_with_stdio(false);
    int n, q;
    cin >> n >> q;
    vector<ll> h(n);
    for (auto& x : h) cin >> x;
    vector<pair<int, int>> qs(q), mq(q);
    for (int i = 0; i < q; i++) cin >> qs[i].first >> qs[i].second, mq[i] = {n - 1 - qs[i].second, n - 1 - qs[i].first};
    auto a = solve(h, qs);
    reverse(h.begin(), h.end());
    auto b = solve(h, mq);
    for (int i = 0; i < q; i++) cout << min(a[i], b[i]) << "\n";
}
