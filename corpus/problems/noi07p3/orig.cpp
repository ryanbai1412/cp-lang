// NOI 2007 Surround (output-only; scored against thresholds on the move count). The wall is
// a minimum vertex cut between the border and the research cells, each cell weighted by a
// per-cell price plus its distance to the nearest battalion (the price rises until the cut
// needs no more battalions than there are). Battalions are matched to the wall greedily by
// distance, improved by pair swaps, then walked in along L-shaped paths: when a path is
// blocked, the blocking battalion goes instead and the rest of the plan is re-matched, so
// the total is never more than the matching's cost. checker.py accepts any valid plan at
// most as long.
#include <bits/stdc++.h>
using namespace std;

struct Dinic {
    struct E { int to; long long cap; };
    vector<E> e;
    vector<vector<int>> g;
    vector<int> lv, it;
    Dinic(int n) : g(n), lv(n), it(n) {}
    void add(int a, int b, long long c) { g[a].push_back(e.size()), e.push_back({b, c}), g[b].push_back(e.size()), e.push_back({a, 0}); }
    bool bfs(int s, int t) {
        fill(lv.begin(), lv.end(), -1);
        queue<int> q;
        lv[s] = 0, q.push(s);
        while (!q.empty()) {
            int x = q.front();
            q.pop();
            for (int id : g[x])
                if (e[id].cap > 0 && lv[e[id].to] < 0) lv[e[id].to] = lv[x] + 1, q.push(e[id].to);
        }
        return lv[t] >= 0;
    }
    long long dfs(int x, int t, long long f) {
        if (x == t) return f;
        for (int& i = it[x]; i < (int)g[x].size(); i++) {
            E& a = e[g[x][i]];
            if (a.cap > 0 && lv[a.to] == lv[x] + 1)
                if (long long r = dfs(a.to, t, min(f, a.cap))) { a.cap -= r, e[g[x][i] ^ 1].cap += r; return r; }
        }
        return 0;
    }
    void flow(int s, int t) {
        while (bfs(s, t)) {
            fill(it.begin(), it.end(), 0);
            while (dfs(s, t, LLONG_MAX)) {}
        }
    }
};

int n, m;
vector<string> g;

int main() {
    int T;
    cin >> T >> n >> m;
    g.resize(n);
    for (auto& r : g) cin >> r;
    int N = n * m;
    auto id = [&](int r, int c) { return r * m + c; };
    vector<int> toks, isO(N);
    for (int r = 0; r < n; r++)
        for (int c = 0; c < m; c++) {
            if (g[r][c] == '#') toks.push_back(id(r, c));
            isO[id(r, c)] = g[r][c] == 'O';
        }
    const int dr[] = {1, -1, 0, 0}, dc[] = {0, 0, 1, -1};
    vector<int> dist(N, INT_MAX);
    {
        queue<int> q;
        for (int x : toks) dist[x] = 0, q.push(x);
        while (!q.empty()) {
            int x = q.front();
            q.pop();
            for (int d = 0; d < 4; d++) {
                int r = x / m + dr[d], c = x % m + dc[d];
                if (r >= 0 && r < n && c >= 0 && c < m && dist[id(r, c)] == INT_MAX) dist[id(r, c)] = dist[x] + 1, q.push(id(r, c));
            }
        }
    }
    vector<int> wall;
    if (count(isO.begin(), isO.end(), 1))
        for (long long price = 1;; price *= 4) {
            const long long INF = LLONG_MAX / 4;
            Dinic f(2 * N + 2);
            int S = 2 * N, Tt = 2 * N + 1;
            for (int r = 0; r < n; r++)
                for (int c = 0; c < m; c++) {
                    int x = id(r, c);
                    f.add(2 * x, 2 * x + 1, isO[x] ? INF : price + (dist[x] == INT_MAX ? N : dist[x]));
                    if (r == 0 || c == 0 || r == n - 1 || c == m - 1) f.add(S, 2 * x, INF);
                    if (isO[x]) f.add(2 * x + 1, Tt, INF);
                    for (int d = 0; d < 4; d++) {
                        int rr = r + dr[d], cc = c + dc[d];
                        if (rr >= 0 && rr < n && cc >= 0 && cc < m) f.add(2 * x + 1, 2 * id(rr, cc), INF);
                    }
                }
            f.flow(S, Tt);
            f.bfs(S, Tt);
            wall.clear();
            for (int x = 0; x < N; x++)
                if (f.lv[2 * x] >= 0 && f.lv[2 * x + 1] < 0) wall.push_back(x);
            if (wall.size() <= toks.size()) break;
        }
    // match: battalions already on the wall stay; the rest go to the nearest free wall cells
    auto md = [&](int a, int b) { return abs(a / m - b / m) + abs(a % m - b % m); };
    vector<int> occ(N, 0), want(N, 0);
    for (int x : toks) occ[x] = 1;
    for (int x : wall) want[x] = 1;
    vector<int> freeTok, openWall;
    for (int x : toks) if (!want[x]) freeTok.push_back(x);
    for (int x : wall) if (!occ[x]) openWall.push_back(x);
    vector<int> src(openWall.size(), -1), taken(N, 0);
    for (int d = 1; d <= 48; d++)
        for (size_t i = 0; i < openWall.size(); i++) {
            if (src[i] >= 0) continue;
            int r0 = openWall[i] / m, c0 = openWall[i] % m;
            for (int a = -d; a <= d && src[i] < 0; a++)
                for (int b : {d - abs(a), abs(a) - d}) {
                    int r = r0 + a, c = c0 + b;
                    if (r >= 0 && r < n && c >= 0 && c < m && occ[id(r, c)] && !want[id(r, c)] && !taken[id(r, c)]) {
                        src[i] = id(r, c), taken[src[i]] = 1;
                        break;
                    }
                }
        }
    for (size_t i = 0; i < openWall.size(); i++)
        if (src[i] < 0) {
            int best = -1;
            for (int x : freeTok)
                if (!taken[x] && (best < 0 || md(x, openWall[i]) < md(best, openWall[i]))) best = x;
            src[i] = best, taken[best] = 1;
        }
    for (int pass = 0, changed = 1; changed && pass < 20; pass++) {
        changed = 0;
        for (size_t i = 0; i < src.size(); i++)
            for (size_t j = i + 1; j < src.size(); j++)
                if (md(src[i], openWall[j]) + md(src[j], openWall[i]) < md(src[i], openWall[i]) + md(src[j], openWall[j]))
                    swap(src[i], src[j]), changed = 1;
    }
    // walk: tgt[x] is where the battalion at x is headed (-1: it stays)
    vector<int> tgt(N, -1);
    for (size_t i = 0; i < src.size(); i++) tgt[src[i]] = openWall[i];
    vector<array<int, 4>> moves;
    for (int s : src)
        while (tgt[s] >= 0) {
            int t = tgt[s];
            vector<int> p{s};
            for (int x = s; x != t;) {
                if (x % m != t % m) x += x % m < t % m ? 1 : -1;
                else x += x / m < t / m ? m : -m;
                p.push_back(x);
            }
            int j = p.size() - 2;
            while (!occ[p[j]]) j--;
            int y = p[j], job = tgt[y];
            for (int k = j; k + 1 < (int)p.size(); k++)
                moves.push_back({p[k] / m + 1, p[k] % m + 1, p[k + 1] / m + 1, p[k + 1] % m + 1});
            occ[y] = 0, occ[t] = 1, tgt[y] = -1, tgt[t] = -1;
            // the battalion at s takes over what the one that moved was doing
            if (j > 0) tgt[s] = job >= 0 ? job : want[y] ? y : -1;
        }
    printf("%zu\n", moves.size());
    for (auto& mv : moves) printf("%d %d %d %d\n", mv[0], mv[1], mv[2], mv[3]);
}
