// noimps23p3: the leaders form a max-weight basis of the matroid "for every department u, at
// most size(u) chosen employees start in u's subtree". Each employee lives over an interval of
// times, so a segment tree over time inserts employees with rollback. Inserting e at x: if no
// ancestor of x is tight (slack 0) e joins; otherwise the circuit is the chosen employees in the
// subtree of the lowest tight ancestor, and e replaces their weakest if it is stronger. Slack is
// kept on heavy paths, the weakest chosen per subtree in a segment tree over the same order.
#include <bits/stdc++.h>
using namespace std;
using ll = long long;

int n, k, m;
vector<int> par, heavy, top, pos, at, sz;

struct Slack {  // range add, rightmost zero (values stay >= 0)
    int N;
    vector<int> mn, lz;
    void build(int o, int l, int r) {
        if (l == r) { mn[o] = sz[at[l]]; return; }
        int md = (l + r) / 2;
        build(2 * o, l, md), build(2 * o + 1, md + 1, r);
        mn[o] = min(mn[2 * o], mn[2 * o + 1]);
    }
    void init() { N = n; mn.assign(4 * N, 0), lz.assign(4 * N, 0); build(1, 0, N - 1); }
    void add(int o, int l, int r, int a, int b, int v) {
        if (b < l || r < a) return;
        if (a <= l && r <= b) { mn[o] += v, lz[o] += v; return; }
        int md = (l + r) / 2;
        add(2 * o, l, md, a, b, v), add(2 * o + 1, md + 1, r, a, b, v);
        mn[o] = min(mn[2 * o], mn[2 * o + 1]) + lz[o];
    }
    int last0(int o, int l, int r, int a, int b, int acc) {  // acc: adds above o
        if (b < l || r < a || mn[o] + acc > 0) return -1;
        if (l == r) return l;
        int md = (l + r) / 2;
        int x = last0(2 * o + 1, md + 1, r, a, b, acc + lz[o]);
        return x >= 0 ? x : last0(2 * o, l, md, a, b, acc + lz[o]);
    }
} slack;

struct MinTree {  // min (value, id) over department positions
    int N;
    vector<pair<int, int>> t;
    void init() { N = 1; while (N < n) N *= 2; t.assign(2 * N, {INT_MAX, -1}); }
    void set(int i, pair<int, int> v) { for (t[i += N] = v; i /= 2;) t[i] = min(t[2 * i], t[2 * i + 1]); }
    pair<int, int> query(int l, int r) {
        pair<int, int> res = {INT_MAX, -1};
        for (l += N, r += N + 1; l < r; l /= 2, r /= 2) {
            if (l & 1) res = min(res, t[l++]);
            if (r & 1) res = min(res, t[--r]);
        }
        return res;
    }
} weakest;

vector<int> ex, ev;
vector<set<pair<int, int>>> here;
ll total = 0;
vector<pair<int, int>> logs;  // (+1 joined | -1 left, employee)

void pathAdd(int x, int v) {
    for (; x >= 0; x = par[top[x]]) slack.add(1, 0, n - 1, pos[top[x]], pos[x], v);
}
void join(int e) {
    pathAdd(ex[e], -1);
    here[ex[e]].insert({ev[e], e});
    weakest.set(pos[ex[e]], *here[ex[e]].begin());
    total += ev[e];
}
void leave(int e) {
    pathAdd(ex[e], 1);
    here[ex[e]].erase({ev[e], e});
    weakest.set(pos[ex[e]], here[ex[e]].empty() ? make_pair(INT_MAX, -1) : *here[ex[e]].begin());
    total -= ev[e];
}
void insert(int e) {
    int u = -1;
    for (int x = ex[e]; x >= 0 && u < 0; x = par[top[x]]) {
        int p = slack.last0(1, 0, n - 1, pos[top[x]], pos[x], 0);
        if (p >= 0) u = at[p];
    }
    if (u >= 0) {
        auto [v, w] = weakest.query(pos[u], pos[u] + sz[u] - 1);
        if (v >= ev[e]) return;
        leave(w), logs.push_back({-1, w});
    }
    join(e), logs.push_back({1, e});
}

vector<vector<int>> seg;
vector<ll> ans;
void put(int o, int l, int r, int a, int b, int e) {
    if (b < l || r < a) return;
    if (a <= l && r <= b) { seg[o].push_back(e); return; }
    int md = (l + r) / 2;
    put(2 * o, l, md, a, b, e), put(2 * o + 1, md + 1, r, a, b, e);
}
void run(int o, int l, int r) {
    size_t mark = logs.size();
    for (int e : seg[o]) insert(e);
    if (l == r) ans[l] = total;
    else {
        int md = (l + r) / 2;
        run(2 * o, l, md), run(2 * o + 1, md + 1, r);
    }
    while (logs.size() > mark) {
        auto [t, e] = logs.back();
        logs.pop_back();
        if (t == 1) leave(e);
        else join(e);
    }
}

int main() {
    int sid;
    scanf("%d %d %d %d", &sid, &n, &k, &m);
    par.assign(n, -1);
    vector<vector<int>> ch(n);
    for (int i = 1; i < n; i++) scanf("%d", &par[i]), par[i]--, ch[par[i]].push_back(i);
    sz.assign(n, 1), heavy.assign(n, -1);
    for (int v = n - 1; v > 0; v--) sz[par[v]] += sz[v];
    for (int v = 0; v < n; v++)
        for (int c : ch[v])
            if (heavy[v] < 0 || sz[c] > sz[heavy[v]]) heavy[v] = c;
    top.assign(n, 0), pos.assign(n, 0), at.assign(n, 0);
    int timer = 0;
    vector<int> st = {0};
    while (!st.empty()) {  // preorder, heavy child first, so subtrees and heavy paths are ranges
        int v = st.back();
        st.pop_back();
        at[pos[v] = timer++] = v;
        for (int c : ch[v])
            if (c != heavy[v]) top[c] = c, st.push_back(c);
        if (heavy[v] >= 0) top[heavy[v]] = top[v], st.push_back(heavy[v]);
    }
    vector<int> from, to;
    for (int i = 0; i < k; i++) {
        int x, v;
        scanf("%d %d", &x, &v);
        ex.push_back(x - 1), ev.push_back(v), from.push_back(0), to.push_back(m);
    }
    for (int t = 1; t <= m; t++) {
        int ty;
        scanf("%d", &ty);
        if (ty == 1) {
            int x, v;
            scanf("%d %d", &x, &v);
            ex.push_back(x - 1), ev.push_back(v), from.push_back(t), to.push_back(m);
        } else {
            int id;
            scanf("%d", &id);
            to[id - 1] = t - 1;
        }
    }
    slack.init(), weakest.init();
    here.assign(n, {});
    seg.assign(4 * (m + 1), {});
    for (size_t e = 0; e < ex.size(); e++)
        if (from[e] <= to[e]) put(1, 0, m, from[e], to[e], e);
    ans.assign(m + 1, 0);
    run(1, 0, m);
    for (int t = 0; t <= m; t++) printf("%lld%c", ans[t], t == m ? '\n' : ' ');
}
