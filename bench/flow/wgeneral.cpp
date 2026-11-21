#include <bits/stdc++.h>
using namespace std;
long long seed = 1;
long long nxt(long long n) { seed = seed * 48271 % 2147483647; return seed % n; }
const int N0 = 800, M0 = 160000;
int n, m, nx;
vector<long long> eu, ev, ew, lab, from_;
vector<int> mate, slack_, st, pa, side_, vis, q;
vector<vector<int>> flower;
size_t at; int stamp_ = 0;
long long delta(int k) { return lab[eu[k]] + lab[ev[k]] - 2 * ew[k]; }
void update_slack(int u, int x) { if (!slack_[x] || delta(u * m + x) < delta(slack_[x] * m + x)) slack_[x] = u; }
void set_slack(int x) { slack_[x] = 0; for (int u = 1; u <= n; u++) if (ew[u * m + x] > 0 && st[u] != x && side_[st[u]] == 0) update_slack(u, x); }
void push(int x) { if (x <= n) q.push_back(x); else for (size_t i = 0; i < flower[x].size(); i++) push(flower[x][i]); }
void set_st(int x, int b) { st[x] = b; if (x > n) for (size_t i = 0; i < flower[x].size(); i++) set_st(flower[x][i], b); }
int get_pr(int b, int xr) {
    int pr = find(flower[b].begin(), flower[b].end(), xr) - flower[b].begin();
    if (pr % 2 == 0) return pr;
    reverse(flower[b].begin() + 1, flower[b].end());
    return (int)flower[b].size() - pr;
}
void set_match(int u, int v) {
    int k = u * m + v; mate[u] = ev[k];
    if (u <= n) return;
    int xr = from_[u * (n + 1) + eu[k]], pr = get_pr(u, xr);
    for (int i = 0; i < pr; i++) set_match(flower[u][i], flower[u][i ^ 1]);
    set_match(xr, v);
    rotate(flower[u].begin(), flower[u].begin() + pr, flower[u].end());
}
void augment(int u, int v) {
    while (true) { int xnv = st[mate[u]]; set_match(u, v); if (!xnv) return; set_match(xnv, st[pa[xnv]]); u = st[pa[xnv]]; v = xnv; }
}
int get_lca(int u, int v) {
    ++stamp_;
    for (; u || v; swap(u, v)) { if (!u) continue; if (vis[u] == stamp_) return u; vis[u] = stamp_; u = st[mate[u]]; if (u) u = st[pa[u]]; }
    return 0;
}
void add_blossom(int u, int lca, int v) {
    int b = n + 1; while (b <= nx && st[b]) b++; if (b > nx) nx++;
    lab[b] = 0; side_[b] = 0; mate[b] = mate[lca];
    flower[b] = {lca};
    for (int x = u, y; x != lca; x = st[pa[y]]) { flower[b].push_back(x); flower[b].push_back(y = st[mate[x]]); push(y); }
    reverse(flower[b].begin() + 1, flower[b].end());
    for (int x = v, y; x != lca; x = st[pa[y]]) { flower[b].push_back(x); flower[b].push_back(y = st[mate[x]]); push(y); }
    set_st(b, b);
    for (int x = 1; x <= nx; x++) ew[b * m + x] = ew[x * m + b] = 0;
    for (int x = 1; x <= n; x++) from_[b * (n + 1) + x] = 0;
    for (size_t i = 0; i < flower[b].size(); i++) {
        int xs = flower[b][i];
        for (int x = 1; x <= nx; x++)
            if (ew[b * m + x] == 0 || delta(xs * m + x) < delta(b * m + x)) {
                eu[b * m + x] = eu[xs * m + x]; ev[b * m + x] = ev[xs * m + x]; ew[b * m + x] = ew[xs * m + x];
                eu[x * m + b] = eu[x * m + xs]; ev[x * m + b] = ev[x * m + xs]; ew[x * m + b] = ew[x * m + xs];
            }
        for (int x = 1; x <= n; x++) if (from_[xs * (n + 1) + x]) from_[b * (n + 1) + x] = xs;
    }
    set_slack(b);
}
void expand_blossom(int b) {
    for (size_t i = 0; i < flower[b].size(); i++) set_st(flower[b][i], flower[b][i]);
    int xr = from_[b * (n + 1) + eu[b * m + pa[b]]], pr = get_pr(b, xr);
    for (int i = 0; i < pr; i += 2) {
        int xs = flower[b][i], xns = flower[b][i + 1];
        pa[xs] = eu[xns * m + xs]; side_[xs] = 1; side_[xns] = 0; slack_[xs] = 0; set_slack(xns); push(xns);
    }
    side_[xr] = 1; pa[xr] = pa[b];
    for (size_t i = pr + 1; i < flower[b].size(); i++) { int xs = flower[b][i]; side_[xs] = -1; set_slack(xs); }
    st[b] = 0;
}
bool on_found(int k) {
    int u = st[eu[k]], v = st[ev[k]];
    if (side_[v] == -1) { pa[v] = eu[k]; side_[v] = 1; int nu = st[mate[v]]; slack_[v] = slack_[nu] = 0; side_[nu] = 0; push(nu); }
    else if (side_[v] == 0) { int lca = get_lca(u, v); if (!lca) { augment(u, v); augment(v, u); return true; } add_blossom(u, lca, v); }
    return false;
}
bool phase() {
    for (int x = 1; x <= nx; x++) { side_[x] = -1; slack_[x] = 0; }
    q.clear(); at = 0;
    for (int x = 1; x <= nx; x++) if (st[x] == x && !mate[x]) { pa[x] = 0; side_[x] = 0; push(x); }
    if (q.empty()) return false;
    while (true) {
        while (at < q.size()) {
            int u = q[at++];
            if (side_[st[u]] == 1) continue;
            for (int v = 1; v <= n; v++) { int k = u * m + v;
                if (ew[k] > 0 && st[u] != st[v]) { if (delta(k) == 0) { if (on_found(k)) return true; } else update_slack(u, st[v]); } }
        }
        long long d = LLONG_MAX;
        for (int b = n + 1; b <= nx; b++) if (st[b] == b && side_[b] == 1) d = min(d, lab[b] / 2);
        for (int x = 1; x <= nx; x++) if (st[x] == x && slack_[x]) {
            if (side_[x] == -1) d = min(d, delta(slack_[x] * m + x)); else if (side_[x] == 0) d = min(d, delta(slack_[x] * m + x) / 2); }
        for (int u = 1; u <= n; u++) if (side_[st[u]] == 0 && lab[u] <= d) return false;
        for (int u = 1; u <= n; u++) { if (side_[st[u]] == 0) lab[u] -= d; else if (side_[st[u]] == 1) lab[u] += d; }
        for (int b = n + 1; b <= nx; b++) if (st[b] == b) { if (side_[b] == 0) lab[b] += 2 * d; else if (side_[b] == 1) lab[b] -= 2 * d; }
        q.clear(); at = 0;
        for (int x = 1; x <= nx; x++)
            if (st[x] == x && slack_[x] && st[slack_[x]] != x && delta(slack_[x] * m + x) == 0)
                if (on_found(slack_[x] * m + x)) return true;
        for (int b = n + 1; b <= nx; b++) if (st[b] == b && side_[b] == 1 && lab[b] == 0) expand_blossom(b);
    }
}
int main() {
    n = N0; m = 2 * n + 1; nx = n;
    eu.assign(m * m, 0); ev.assign(m * m, 0); ew.assign(m * m, 0);
    for (int u = 1; u <= n; u++) for (int v = 1; v <= n; v++) { eu[u * m + v] = u; ev[u * m + v] = v; }
    long long top = 0;
    for (int i = 0; i < M0; i++) {
        int a = nxt(N0) + 1, b = nxt(N0) + 1; long long w = nxt(1000000);
        if (a != b && w > ew[a * m + b]) { ew[a * m + b] = ew[b * m + a] = w; top = max(top, w); }
    }
    from_.assign(m * (n + 1), 0);
    for (int u = 1; u <= n; u++) from_[u * (n + 1) + u] = u;
    lab.assign(m, 0); for (int i = 1; i <= n; i++) lab[i] = top;
    mate.assign(m, 0); slack_.assign(m, 0); st.resize(m); iota(st.begin(), st.end(), 0);
    pa.assign(m, 0); side_.assign(m, -1); vis.assign(m, 0); flower.assign(m, {});
    while (phase()) {}
    long long total = 0;
    for (int u = 1; u <= n; u++) if (mate[u] > u) total += ew[u * m + mate[u]];
    printf("%lld\n", total);
}
