// COCI 2021/2022 Round 6 #4 (Palindromi). Every string built along the way is a block of the
// final string, so the task is distinct palindromes in n - 1 offline ranges [l, r]. Sweep r over
// the final string's palindromic tree; count each distinct palindrome at the start of its last
// occurrence (a Fenwick tree over starts). The palindromic suffixes ending at r split into
// O(log n) series with one difference d; inside a series every member but the longest last
// occurred d earlier, so the series' updates telescope to +1 at the shortest member's new start
// and -1 at the longest member's previous start, which is the latest time the suffix-link
// subtree of the longest member was the longest palindromic suffix (a max segment tree).
#include <bits/stdc++.h>
using namespace std;
int main() {
    int n;
    scanf("%d", &n);
    static char buf[100005];
    scanf("%s", buf);
    vector<int> par(2 * n), lc(2 * n, -1), rc(2 * n, -1), sz(2 * n, 1);
    iota(par.begin(), par.end(), 0);
    function<int(int)> find = [&](int x) { while (par[x] != x) x = par[x] = par[par[x]]; return x; };
    vector<int> root(n);
    iota(root.begin(), root.end(), 0);
    vector<int> node(n - 1);
    for (int i = 0; i < n - 1; i++) {
        int a, b;
        scanf("%d %d", &a, &b);
        int x = find(a - 1), y = find(b - 1), v = n + i;
        lc[v] = root[x], rc[v] = root[y];
        sz[v] = sz[root[x]] + sz[root[y]];
        par[y] = x;
        root[x] = v;
        node[i] = v;
    }
    int top = n == 1 ? 0 : 2 * n - 2;
    vector<int> start(2 * n, 0);
    string s(n, '?');
    vector<int> st = {top};
    while (!st.empty()) {
        int v = st.back();
        st.pop_back();
        if (v < n) { s[start[v]] = buf[v]; continue; }
        start[lc[v]] = start[v], start[rc[v]] = start[v] + sz[lc[v]];
        st.push_back(lc[v]), st.push_back(rc[v]);
    }
    vector<vector<pair<int, int>>> at(n);  // queries by right end: (left, index)
    for (int i = 0; i < n - 1; i++) at[start[node[i]] + sz[node[i]] - 1].push_back({start[node[i]], i});
    // Palindromic tree: node 0 has length -1, node 1 length 0.
    vector<int> len = {-1, 0}, link = {0, 0}, diff = {0, 0}, slink = {0, 0};
    vector<array<int, 2>> nx = {{-1, -1}, {-1, -1}};
    vector<int> suf(n);
    int last = 1;
    for (int i = 0; i < n; i++) {
        int c = s[i] - '0';
        auto ok = [&](int v) { return i - len[v] - 1 >= 0 && s[i - len[v] - 1] == s[i]; };
        while (!ok(last)) last = link[last];
        if (nx[last][c] < 0) {
            int v = len.size();
            len.push_back(len[last] + 2);
            int w = link[last];
            if (len[v] == 1) w = 1;
            else { while (!ok(w)) w = link[w]; w = nx[w][c]; }
            link.push_back(w);
            diff.push_back(len[v] - len[w]);
            slink.push_back(diff[v] == diff[w] ? slink[w] : w);
            nx.push_back({-1, -1});
            nx[last][c] = v;
        }
        last = nx[last][c];
        suf[i] = last;
    }
    int m = len.size();
    vector<vector<int>> kids(m);
    for (int v = 2; v < m; v++) kids[link[v]].push_back(v);
    vector<int> tin(m), tout(m);
    int timer = 0;
    vector<pair<int, int>> dfs = {{1, 0}};
    while (!dfs.empty()) {
        auto& [v, k] = dfs.back();
        if (k == 0) tin[v] = timer++;
        if (k < (int)kids[v].size()) { int w = kids[v][k++]; dfs.push_back({w, 0}); }
        else { tout[v] = timer; dfs.pop_back(); }
    }
    int S = 1;
    while (S < timer) S *= 2;
    vector<int> seg(2 * S, -1);
    auto put = [&](int p, int val) { for (seg[p += S] = val; p > 1; p /= 2) seg[p / 2] = max(seg[p], seg[p ^ 1]); };
    auto qmax = [&](int l, int r) {
        int res = -1;
        for (l += S, r += S; l < r; l /= 2, r /= 2) {
            if (l & 1) res = max(res, seg[l++]);
            if (r & 1) res = max(res, seg[--r]);
        }
        return res;
    };
    vector<int> fen(n + 1, 0);
    auto fadd = [&](int p, int d) { for (p++; p <= n; p += p & -p) fen[p] += d; };
    auto fsum = [&](int p) { int t = 0; for (; p > 0; p -= p & -p) t += fen[p]; return t; };  // starts < p
    int total = 0;
    vector<int> ans(n - 1);
    for (int i = 0; i < n; i++) {
        for (int v = suf[i]; len[v] > 0; v = slink[v]) {
            int shortest = len[slink[v]] + diff[v];
            fadd(i - shortest + 1, 1), total++;
            int prev = qmax(tin[v], tout[v]);
            if (prev >= 0) fadd(prev - len[v] + 1, -1), total--;
        }
        put(tin[suf[i]], i);
        for (auto [l, idx] : at[i]) ans[idx] = total - fsum(l);
    }
    for (int x : ans) printf("%d\n", x);
}
