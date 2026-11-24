// The answer is the least w whose graph of pairs at squared distance <= w is not bipartite
// (official editorial). If some point has 6 others within distance r, two of them make a
// triangle of sides <= r, so w <= R = min over points of the 6th-nearest distance; and below
// R every point has at most 5 neighbours, all among its 6 nearest. So Kruskal with a parity
// DSU over the 6-nearest-neighbour edges shorter than R finds w, or w = R. Exact kNN comes
// from a k-d tree; squared distances fit in int64 for |coordinates| <= 1e9.
#include <bits/stdc++.h>
using namespace std;
typedef long long ll;
struct Pt { ll x, y; int id; };
int n, K;
vector<Pt> t;
vector<char> ax;
void build(int l, int r) {
    if (r - l <= 1) return;
    ll mnx = LLONG_MAX, mxx = LLONG_MIN, mny = LLONG_MAX, mxy = LLONG_MIN;
    for (int i = l; i < r; i++)
        mnx = min(mnx, t[i].x), mxx = max(mxx, t[i].x), mny = min(mny, t[i].y), mxy = max(mxy, t[i].y);
    int m = (l + r) / 2;
    char a = (mxx - mnx >= mxy - mny) ? 0 : 1;
    ax[m] = a;
    nth_element(t.begin() + l, t.begin() + m, t.begin() + r, [a](const Pt& p, const Pt& q) {
        return a ? p.y < q.y : p.x < q.x;
    });
    build(l, m), build(m + 1, r);
}
priority_queue<pair<ll, int>> heap;
void query(int l, int r, const Pt& q) {
    if (l >= r) return;
    int m = (l + r) / 2;
    const Pt& p = t[m];
    if (p.id != q.id) {
        ll dx = p.x - q.x, dy = p.y - q.y, d = dx * dx + dy * dy;
        if ((int)heap.size() < K) heap.push({d, p.id});
        else if (d < heap.top().first) heap.pop(), heap.push({d, p.id});
    }
    if (r - l == 1) return;
    ll diff = ax[m] ? q.y - p.y : q.x - p.x;
    bool leftFirst = diff < 0;
    if (leftFirst) query(l, m, q); else query(m + 1, r, q);
    if ((int)heap.size() < K || diff * diff < heap.top().first) {
        if (leftFirst) query(m + 1, r, q); else query(l, m, q);
    }
}
vector<int> par, parity;
pair<int, int> find(int v) {
    int p = 0;
    while (par[v] != v) p ^= parity[v], v = par[v];
    return {v, p};
}
int main() {
    scanf("%d", &n);
    t.resize(n), ax.assign(n, 0);
    for (int i = 0; i < n; i++) scanf("%lld %lld", &t[i].x, &t[i].y), t[i].id = i;
    vector<Pt> pts = t;
    K = min(6, n - 1);
    build(0, n);
    vector<vector<pair<ll, int>>> nn(n);
    ll R = LLONG_MAX;
    for (int i = 0; i < n; i++) {
        query(0, n, pts[i]);
        if ((int)heap.size() == 6) R = min(R, heap.top().first);
        while (!heap.empty()) nn[i].push_back(heap.top()), heap.pop();
    }
    vector<tuple<ll, int, int>> edges;
    for (int i = 0; i < n; i++)
        for (auto [d, j] : nn[i])
            if (d < R && i < j) edges.push_back({d, i, j});
            else if (d < R && i > j) edges.push_back({d, j, i});
    sort(edges.begin(), edges.end());
    edges.erase(unique(edges.begin(), edges.end()), edges.end());
    par.resize(n), parity.assign(n, 0);
    iota(par.begin(), par.end(), 0);
    vector<int> sz(n, 1);
    for (auto [d, a, b] : edges) {
        auto [ra, pa] = find(a);
        auto [rb, pb] = find(b);
        if (ra == rb) {
            if (pa == pb) { printf("%lld\n", d); return 0; }
            continue;
        }
        if (sz[ra] < sz[rb]) swap(ra, rb);
        par[rb] = ra, parity[rb] = pa ^ pb ^ 1, sz[ra] += sz[rb];
    }
    printf("%lld\n", R);
}
