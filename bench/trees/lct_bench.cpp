#include <bits/stdc++.h>
using namespace std;
int n;
vector<int> lc, rc, pa, stk;
vector<char> flip;
vector<long long> val, agg;
bool top(int x) { int p = pa[x]; return !p || (lc[p] != x && rc[p] != x); }
void turn(int x) { flip[x] ^= 1; }
void push(int x) {
    if (flip[x]) { swap(lc[x], rc[x]); if (lc[x]) turn(lc[x]); if (rc[x]) turn(rc[x]); flip[x] = 0; }
}
void pull(int x) { agg[x] = agg[lc[x]] + val[x] + agg[rc[x]]; }
void rotate(int x) {
    int p = pa[x], g = pa[p]; bool pt = top(p);
    if (lc[p] == x) { lc[p] = rc[x]; if (rc[x]) pa[rc[x]] = p; rc[x] = p; }
    else { rc[p] = lc[x]; if (lc[x]) pa[lc[x]] = p; lc[x] = p; }
    pa[p] = x; pa[x] = g;
    if (!pt) { if (lc[g] == p) lc[g] = x; else rc[g] = x; }
    pull(p); pull(x);
}
void splay(int x) {
    stk.clear(); int y = x; stk.push_back(y);
    while (!top(y)) { y = pa[y]; stk.push_back(y); }
    while (!stk.empty()) { push(stk.back()); stk.pop_back(); }
    while (!top(x)) {
        int p = pa[x];
        if (!top(p)) { int g = pa[p]; if ((lc[g] == p) == (lc[p] == x)) rotate(p); else rotate(x); }
        rotate(x);
    }
}
int access(int x) {
    int last = 0, joined = x;
    for (int u = x; u; u = pa[u]) { splay(u); rc[u] = last; pull(u); last = u; joined = u; }
    splay(x); return joined;
}
void make_root(int v) { access(v); turn(v); }
void link(int u, int v) { make_root(u); pa[u] = v; }
void cut(int u, int v) { make_root(u); access(v); lc[v] = 0; pa[u] = 0; pull(v); }
int main() {
    scanf("%d", &n);
    lc.assign(n + 1, 0); rc = lc; pa = lc; flip.assign(n + 1, 0); val.assign(n + 1, 0);
    for (int i = 1; i <= n; i++) scanf("%lld", &val[i]);
    agg = val; agg[0] = 0;
    for (int v = 2; v <= n; v++) { int p; scanf("%d", &p); link(v, p); }
    int q; scanf("%d", &q);
    long long hash = 0;
    while (q--) {
        int t; scanf("%d", &t);
        if (t == 0) { int v; long long x; scanf("%d %lld", &v, &x); access(v); val[v] = x; pull(v); }
        else if (t == 1) { int v, o, p; scanf("%d %d %d", &v, &o, &p); cut(v, o); link(v, p); }
        else { int u, v; scanf("%d %d", &u, &v); make_root(u); access(v);
            hash = ((hash * 31 + agg[v]) % 1000000007 + 1000000007) % 1000000007; }
    }
    printf("%lld\n", hash);
}
