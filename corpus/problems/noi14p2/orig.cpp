// NOI 2014 Magic Forest reference: edges by increasing a, a link-cut tree keeps
// the minimum spanning forest on b; after each edge, a_i + (max b on the 1-n path).
#include <bits/stdc++.h>
using namespace std;

struct LCT {
    vector<int> ch0, ch1, par, val, mx, rev;
    LCT(int n) : ch0(n), ch1(n), par(n), val(n), mx(n), rev(n) { iota(mx.begin(), mx.end(), 0); }
    bool root(int x) { int p = par[x]; return !p || (ch0[p] != x && ch1[p] != x); }
    int best(int a, int b) { return val[a] >= val[b] ? a : b; }
    void pull(int x) {
        mx[x] = x;
        if (ch0[x]) mx[x] = best(mx[x], mx[ch0[x]]);
        if (ch1[x]) mx[x] = best(mx[x], mx[ch1[x]]);
    }
    void push(int x) {
        if (!rev[x]) return;
        for (int c : {ch0[x], ch1[x]}) if (c) { swap(ch0[c], ch1[c]); rev[c] ^= 1; }
        rev[x] = 0;
    }
    void rotate(int x) {
        int p = par[x], g = par[p];
        bool left = ch0[p] == x;
        if (!root(p)) (ch0[g] == p ? ch0[g] : ch1[g]) = x;
        par[x] = g;
        int b = left ? ch1[x] : ch0[x];
        (left ? ch0[p] : ch1[p]) = b;
        if (b) par[b] = p;
        (left ? ch1[x] : ch0[x]) = p;
        par[p] = x;
        pull(p), pull(x);
    }
    void splay(int x) {
        vector<int> st{x};
        for (int y = x; !root(y); y = par[y]) st.push_back(par[y]);
        for (int i = st.size() - 1; i >= 0; i--) push(st[i]);
        while (!root(x)) {
            int p = par[x];
            if (!root(p)) rotate((ch0[p] == x) == (ch0[par[p]] == p) ? p : x);
            rotate(x);
        }
    }
    void access(int x) {
        for (int last = 0; x; last = x, x = par[x]) { splay(x); ch1[x] = last; pull(x); }
    }
    void makeroot(int x) { access(x); splay(x); swap(ch0[x], ch1[x]); rev[x] ^= 1; }
    int findroot(int x) {
        access(x); splay(x);
        while (true) { push(x); if (!ch0[x]) break; x = ch0[x]; }
        splay(x);
        return x;
    }
    int pathmax(int x, int y) { makeroot(x); access(y); splay(y); return mx[y]; }
    void link(int x, int y) { makeroot(x); par[x] = y; }
    void cut(int x, int y) { makeroot(x); access(y); splay(y); ch0[y] = 0; par[x] = 0; pull(y); }
};

int main() {
    int n, m;
    scanf("%d %d", &n, &m);
    vector<array<int, 4>> e(m);
    for (auto& [a, b, x, y] : e) scanf("%d %d %d %d", &x, &y, &a, &b);
    sort(e.begin(), e.end());
    LCT t(n + m + 1);
    for (int i = 0; i < m; i++) t.val[n + 1 + i] = e[i][1];
    long long ans = LLONG_MAX;
    for (int i = 0; i < m; i++) {
        auto [a, b, x, y] = e[i];
        int me = n + 1 + i;
        if (x != y) {
            if (t.findroot(x) != t.findroot(y)) {
                t.link(x, me), t.link(me, y);
            } else {
                int w = t.pathmax(x, y);
                if (t.val[w] > b) {
                    int j = w - n - 1;
                    t.cut(e[j][2], w), t.cut(w, e[j][3]);
                    t.link(x, me), t.link(me, y);
                }
            }
        }
        if (t.findroot(1) == t.findroot(n)) ans = min(ans, (long long)a + t.val[t.pathmax(1, n)]);
    }
    printf("%lld\n", ans == LLONG_MAX ? -1 : ans);
}
