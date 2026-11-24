// Reference for DMOJ oly23practice7 (no accepted submission: its C++ submissions are WA or RTE;
// written for the corpus). Tree edges 1..n-1 form the MST iff every non-tree edge outweighs each
// tree edge on its tree path. Tighten the ranges along those precedences: a tree edge's r drops
// below every covering non-tree edge's r (paint paths in increasing r with a jump-to-unpainted
// DSU), and a non-tree edge's l rises above the l of every tree edge on its path (binary lifting
// path max). Then the earliest-deadline-first schedule over weights 1..m respects every
// precedence (the earlier edge has the earlier release and the earlier deadline), so it finds
// an assignment iff one exists.
#include <bits/stdc++.h>
using namespace std;

static char buf[1 << 25];
int bl = 0, bp = 0;
int gc() {
    if (bp == bl) { bl = fread(buf, 1, sizeof buf, stdin); bp = 0; if (bl <= 0) return -1; }
    return buf[bp++];
}
int rd() {
    int c = gc();
    while (c < '0' || c > '9') c = gc();
    int x = 0;
    for (; c >= '0' && c <= '9'; c = gc()) x = x * 10 + (c - '0');
    return x;
}

int main() {
    int t = rd();
    string out;
    while (t--) {
        int n = rd(), m = rd();
        vector<int> u(m), v(m), l(m), r(m);
        for (int i = 0; i < m; i++) u[i] = rd() - 1, v[i] = rd() - 1, l[i] = rd(), r[i] = rd();
        vector<vector<pair<int, int>>> adj(n);
        for (int i = 0; i < n - 1; i++) adj[u[i]].push_back({v[i], i}), adj[v[i]].push_back({u[i], i});
        int LOG = 1;
        while ((1 << LOG) < n) LOG++;
        vector<int> dep(n, 0), pe(n, -1), order{0};
        vector<vector<int>> up(LOG, vector<int>(n, 0)), mx(LOG, vector<int>(n, 0));
        vector<char> seen(n, 0);
        seen[0] = 1;
        for (size_t k = 0; k < order.size(); k++) {
            int a = order[k];
            for (auto [b, e] : adj[a])
                if (!seen[b]) seen[b] = 1, dep[b] = dep[a] + 1, up[0][b] = a, pe[b] = e, order.push_back(b);
        }
        // Painting: r_tree = min(r_tree, min r_e - 1) over covering non-tree edges e.
        vector<int> nt;
        for (int i = n - 1; i < m; i++) nt.push_back(i);
        sort(nt.begin(), nt.end(), [&](int a, int b) { return r[a] < r[b]; });
        vector<int> jump(n);  // nearest ancestor-or-self whose parent edge is unpainted
        iota(jump.begin(), jump.end(), 0);
        function<int(int)> find = [&](int x) {
            while (jump[x] != x) jump[x] = jump[jump[x]], x = jump[x];
            return x;
        };
        auto lca = [&](int a, int b) {
            if (dep[a] < dep[b]) swap(a, b);
            for (int k = LOG - 1; k >= 0; k--)
                if (dep[a] - (1 << k) >= dep[b]) a = up[k][a];
            if (a == b) return a;
            for (int k = LOG - 1; k >= 0; k--)
                if (up[k][a] != up[k][b]) a = up[k][a], b = up[k][b];
            return up[0][a];
        };
        for (int k = 1; k < LOG; k++)
            for (int x = 0; x < n; x++) up[k][x] = up[k - 1][up[k - 1][x]];
        for (int e : nt) {
            int c = lca(u[e], v[e]);
            for (int a : {u[e], v[e]})
                for (int x = find(a); dep[x] > dep[c]; x = find(x)) {
                    r[pe[x]] = min(r[pe[x]], r[e] - 1);
                    jump[x] = up[0][x];
                }
        }
        // Path max of tightened tree l.
        for (int x = 1; x < n; x++) mx[0][order[x]] = l[pe[order[x]]];
        for (int k = 1; k < LOG; k++)
            for (int x = 0; x < n; x++) mx[k][x] = max(mx[k - 1][x], mx[k - 1][up[k - 1][x]]);
        auto pathmax = [&](int a, int b) {
            int res = 0;
            if (dep[a] < dep[b]) swap(a, b);
            for (int k = LOG - 1; k >= 0; k--)
                if (dep[a] - (1 << k) >= dep[b]) res = max(res, mx[k][a]), a = up[k][a];
            if (a == b) return res;
            for (int k = LOG - 1; k >= 0; k--)
                if (up[k][a] != up[k][b]) res = max({res, mx[k][a], mx[k][b]}), a = up[k][a], b = up[k][b];
            return max({res, mx[0][a], mx[0][b]});
        };
        for (int e = n - 1; e < m; e++) l[e] = max(l[e], pathmax(u[e], v[e]) + 1);
        // Earliest deadline first.
        vector<vector<int>> rel(m + 2);
        bool ok = true;
        for (int i = 0; i < m; i++) {
            if (l[i] > r[i]) ok = false;
            else rel[l[i]].push_back(i);
        }
        vector<int> w(m, 0);
        priority_queue<pair<int, int>, vector<pair<int, int>>, greater<>> pq;
        for (int x = 1; x <= m && ok; x++) {
            for (int i : rel[x]) pq.push({r[i], i});
            if (pq.empty() || pq.top().first < x) { ok = false; break; }
            w[pq.top().second] = x;
            pq.pop();
        }
        if (!ok) { out += "NO\n"; continue; }
        out += "YES\n";
        for (int i = 0; i < m; i++) out += to_string(w[i]) + (i + 1 < m ? ' ' : '\n');
    }
    fputs(out.c_str(), stdout);
}
