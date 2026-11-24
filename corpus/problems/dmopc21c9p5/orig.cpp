// Placing card k at position p is consistent with a exactly when (1) k >= L_p, the larger
// report of p's two neighbours, and (2) if someone reports k, p neighbours every one of them.
// These are per (card, position) conditions, so the arrangements are the perfect matchings
// of that bipartite graph (official editorial, subtask 2). A card is fixed when its matched
// edge is its only edge on an alternating cycle: the SCCs of "card -> card holding a position
// it accepts" (Tassa). Unreported cards accept a prefix of the positions sorted by L, so
// those arcs go through a chain of auxiliary nodes and the graph stays linear.
#include <bits/stdc++.h>
using namespace std;
static char ibuf[1 << 25];
int ipos = 0, ilen = 0;
int gc() {
    if (ipos == ilen) ilen = fread(ibuf, 1, sizeof ibuf, stdin), ipos = 0;
    return ipos < ilen ? ibuf[ipos++] : -1;
}
int rd() {
    int c = gc();
    while (c < '0') c = gc();
    int x = 0;
    while (c >= '0') x = x * 10 + c - '0', c = gc();
    return x;
}
int main() {
    int T = rd();
    string out;
    while (T--) {
        int n = rd();
        vector<int> a(n), L(n);
        for (auto& x : a) x = rd();
        for (int p = 0; p < n; p++) L[p] = max(a[(p + n - 1) % n], a[(p + 1) % n]);
        // opts[k]: explicit positions for a reported card; unreported cards use the prefix.
        vector<vector<int>> claim(n + 1);
        for (int i = 0; i < n; i++) claim[a[i]].push_back(i);
        vector<vector<int>> opts(n + 1);
        vector<char> reported(n + 1, 0);
        for (int k = 1; k <= n; k++) {
            if (claim[k].empty()) continue;
            reported[k] = 1;
            vector<int> cand = {(claim[k][0] + n - 1) % n, (claim[k][0] + 1) % n};
            for (int p : cand) {
                bool ok = L[p] <= k;
                for (int i : claim[k]) ok = ok && ((p + 1) % n == i || (p + n - 1) % n == i);
                if (ok && find(opts[k].begin(), opts[k].end(), p) == opts[k].end()) opts[k].push_back(p);
            }
        }
        vector<int> ord(n);
        iota(ord.begin(), ord.end(), 0);
        sort(ord.begin(), ord.end(), [&](int x, int y) { return L[x] < L[y]; });
        vector<int> rk(n), sortedL(n);
        for (int t = 0; t < n; t++) rk[ord[t]] = t, sortedL[t] = L[ord[t]];
        auto thr = [&](int k) { return int(upper_bound(sortedL.begin(), sortedL.end(), k) - sortedL.begin()); };
        vector<int> cardAt(n, -1), posOf(n + 1, -1);
        // Greedy start: reported cards (fewest options first), then unreported cards in
        // increasing order against positions in increasing L.
        vector<int> rep;
        for (int k = 1; k <= n; k++)
            if (reported[k]) rep.push_back(k);
        stable_sort(rep.begin(), rep.end(), [&](int x, int y) { return opts[x].size() < opts[y].size(); });
        for (int k : rep) {
            int best = -1;
            for (int p : opts[k])
                if (cardAt[p] < 0 && (best < 0 || L[p] > L[best])) best = p;
            if (best >= 0) cardAt[best] = k, posOf[k] = best;
        }
        int ptr = 0;
        for (int k = 1; k <= n; k++) {
            if (reported[k]) continue;
            while (ptr < n && cardAt[ord[ptr]] >= 0) ptr++;
            if (ptr < n && sortedL[ptr] <= k) cardAt[ord[ptr]] = k, posOf[k] = ord[ptr], ptr++;
        }
        // Augment the rest by BFS; nxt[] skips already visited ranks in the sorted order.
        vector<int> nxt(n + 1), parPos(n + 1), parCard(n), seenPos(n, 0), seenCard(n + 1, 0);
        int stamp = 0;
        function<int(int)> fnd = [&](int x) {
            while (nxt[x] != x) nxt[x] = nxt[nxt[x]], x = nxt[x];
            return x;
        };
        for (int s = 1; s <= n; s++) {
            if (posOf[s] >= 0) continue;
            ++stamp;
            iota(nxt.begin(), nxt.end(), 0);
            deque<int> q = {s};
            seenCard[s] = stamp;
            int endPos = -1;
            while (!q.empty() && endPos < 0) {
                int k = q.front();
                q.pop_front();
                auto visit = [&](int p) {
                    seenPos[p] = stamp, parCard[p] = k;
                    nxt[rk[p]] = rk[p] + 1;
                    int c = cardAt[p];
                    if (c < 0) { endPos = p; return; }
                    if (seenCard[c] != stamp) seenCard[c] = stamp, q.push_back(c);
                };
                if (reported[k]) {
                    for (int p : opts[k])
                        if (seenPos[p] != stamp && endPos < 0) visit(p);
                } else {
                    int lim = thr(k);
                    for (int t = fnd(0); t < lim && endPos < 0; t = fnd(t)) visit(ord[t]);
                }
            }
            for (int p = endPos; p >= 0;) {
                int k = parCard[p], prev = posOf[k];
                cardAt[p] = k, posOf[k] = p;
                p = prev;
            }
        }
        // Nodes: cards 1..n, aux chain n+1..2n (aux for rank t is n+1+t).
        int V = 2 * n + 1;
        vector<vector<int>> g(V);
        for (int k = 1; k <= n; k++) {
            if (reported[k]) {
                for (int p : opts[k]) g[k].push_back(cardAt[p]);
            } else {
                int lim = thr(k);
                if (lim > 0) g[k].push_back(n + lim);
            }
        }
        for (int t = 0; t < n; t++) {
            g[n + 1 + t].push_back(cardAt[ord[t]]);
            if (t) g[n + 1 + t].push_back(n + t);
        }
        vector<int> idx(V, -1), low(V), comp(V, -1), stk, it(V, 0);
        vector<char> on(V, 0);
        int cnt = 0, nc = 0;
        for (int r = 1; r < V; r++) {
            if (idx[r] >= 0) continue;
            vector<int> cs = {r};
            idx[r] = low[r] = cnt++, stk.push_back(r), on[r] = 1;
            while (!cs.empty()) {
                int v = cs.back();
                if (it[v] < (int)g[v].size()) {
                    int w = g[v][it[v]++];
                    if (idx[w] < 0) idx[w] = low[w] = cnt++, stk.push_back(w), on[w] = 1, cs.push_back(w);
                    else if (on[w]) low[v] = min(low[v], idx[w]);
                } else {
                    cs.pop_back();
                    if (!cs.empty()) low[cs.back()] = min(low[cs.back()], low[v]);
                    if (low[v] == idx[v]) {
                        int w;
                        do w = stk.back(), stk.pop_back(), on[w] = 0, comp[w] = nc;
                        while (w != v);
                        nc++;
                    }
                }
            }
        }
        // Per component, the two smallest-L positions whose holder is in it.
        vector<array<int, 2>> best(nc, {-1, -1});
        for (int t = 0; t < n; t++) {
            int p = ord[t], c = comp[cardAt[p]];
            if (best[c][0] < 0) best[c][0] = p;
            else if (best[c][1] < 0) best[c][1] = p;
        }
        vector<int> res(n, 0);
        for (int k = 1; k <= n; k++) {
            int p = posOf[k], c = comp[k];
            bool other = false;
            if (reported[k]) {
                for (int q2 : opts[k]) other = other || (q2 != p && comp[cardAt[q2]] == c);
            } else {
                int q2 = best[c][0] == p ? best[c][1] : best[c][0];
                other = q2 >= 0 && L[q2] <= k;
            }
            if (!other) res[p] = k;
        }
        for (int i = 0; i < n; i++) out += to_string(res[i]), out += i + 1 < n ? ' ' : '\n';
    }
    fwrite(out.data(), 1, out.size(), stdout);
}
