#include "bits/extc++.h"

using namespace std;

template <typename T>
void dbgh(const T& t) {
    cerr << t << endl;
}

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t << " | ";
    dbgh(u...);
}

#ifdef DEBUG
#define dbg(...)                                           \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" \
         << ": ";                                          \
    dbgh(__VA_ARGS__)
#else
#define cerr   \
    if (false) \
    cerr
#define dbg(...)
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

template <typename T>
using min_pq = priority_queue<T, vector<T>, greater<T>>;

template <typename Cb>
struct KeyCmp {
    Cb cb;

    KeyCmp(Cb cb) : cb(cb) {}

    template <typename T>
    bool operator()(const T& a, const T& b) const {
        return cb(a) < cb(b);
    }
};

struct DSU {
    vector<int> p;
    vector<long> vmn;

    DSU(const vector<long>& arr) : p(sz(arr), -1), vmn(arr) {}

    int find(int u) {
        return p[u] < 0 ? u : find(p[u]);
    }

    long query(int u) {
        return vmn[find(u)];
    }

    bool merge(int u, int v) {
        u = find(u);
        v = find(v);
        if (u == v) {
            return false;
        }
        if (p[u] < p[v]) {
            swap(u, v);
        }
        p[v] += p[u];
        p[u] = v;
        vmn[v] = min(vmn[u], vmn[v]);
        return true;
    }
};

void solve() {
    int n, m;
    cin >> n >> m;
    vector<pair<int, int>> graph[n];
    array<int, 4> edges[m];
    for (auto& [u, v, l, w] : edges) {
        cin >> u >> v >> l >> w;
        u--;
        v--;
        graph[u].emplace_back(v, l);
        graph[v].emplace_back(u, l);
    }
    sort(edges, edges + m,
         KeyCmp {[&](const auto& a) -> int { return -a[3]; }});
    vector<int> ews;
    for (auto& a : edges) {
        ews.push_back(a[3]);
    }
    vector<long> dist(n, 1e18);
    {
        min_pq<pair<long, int>> pq;
        auto push = [&](int u, long d) -> void {
            if (d >= dist[u]) {
                return;
            }
            dist[u] = d;
            pq.emplace(d, u);
        };
        push(0, 0);
        while (sz(pq)) {
            auto [d, u] = pq.top();
            pq.pop();
            if (d != dist[u]) {
                continue;
            }
            for (auto& [v, w] : graph[u]) {
                push(v, d + w);
            }
        }
    }
    int q, online, tmax, last_ans = 0;
    cin >> q >> online >> tmax;
    while (q--) {
        int u, t;
        cin >> u >> t;
        u = (u + online * last_ans - 1) % n;
        t = (t + online * last_ans) % (tmax + 1);
        int tm = int(lower_bound(begin(ews), end(ews), t, greater<>()) - ews.begin());
        DSU dsu(dist);
        for (int i = 0; i < tm; i++) {
            dsu.merge(edges[i][0], edges[i][1]);
        }
        last_ans = dsu.query(u);
        cout << last_ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}