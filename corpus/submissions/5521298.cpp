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

struct ST {
    struct Node {
        int lc, rc;
    };

    vector<Node> heap {{}};
    int n, root = 0;

    ST(int n) : n(n) {}

    int alloc() {
        heap.push_back({});
        return sz(heap) - 1;
    }

    void update(int& o, int l, int r, int ind, int x) {
        if (l == r) {
            o = x;
            return;
        }
        heap.push_back(heap[o]);
        o = sz(heap) - 1;
        int mid = (l + r) / 2;
        if (ind <= mid) {
            update(heap[o].lc, l, mid, ind, x);
        } else{
            update(heap[o].rc, mid + 1, r, ind, x);
        }
    }

    void update(int ind, int x) {
        if (heap.size() + 100 >= heap.capacity()) {
            heap.reserve(max(heap.capacity() + 100, heap.capacity() * 2));
        }
        update(root, 0, n - 1, ind, x);
    }

    int query(int o, int l, int r, int ind) const {
        if (l == r) {
            return o;
        }
        assert(o);
        int mid = (l + r) / 2;
        if (ind <= mid) {
            return query(heap[o].lc, l, mid, ind);
        } else {
            return query(heap[o].rc, mid + 1, r, ind);
        }
    }

    int query(int ind) const {
        return query(root, 0, n - 1, ind);
    }
};

struct DSU {
    int n;
    ST p, vmn;
    vector<pair<int, int>> hist;

    DSU(const vector<int>& arr) : n(sz(arr)), p(n), vmn(n) {
        for (int i = 0; i < n; i++) {
            p.update(i, -1);
            vmn.update(i, arr[i]);
        }
    }

    int find(int u) {
        int pu = p.query(u);
        return pu < 0 ? u : find(pu);
    }

    int query(int u) {
        return vmn.query(find(u));
    }

    bool merge(int u, int v) {
        u = find(u);
        v = find(v);
        if (u == v) {
            return false;
        }
        int pu = p.query(u), pv = p.query(v);
        if (pu < pv) {
            swap(u, v);
        }
        p.update(v, pu + pv);
        p.update(u, v);
        vmn.update(v, min(vmn.query(u), vmn.query(v)));
        return true;
    }

    void save() {
        hist.emplace_back(p.root, vmn.root);
    }

    void rollback(int ind) {
        auto& [pr, vmnr] = hist[ind];
        p.root = pr;
        vmn.root = vmnr;
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
    vector<int> dist(n, INT_MAX);
    {
        min_pq<pair<int, int>> pq;
        auto push = [&](int u, int d) -> void {
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
    DSU dsu(dist);
    dsu.save();
    for (auto& [u, v, _l, _w] : edges) {
        dsu.merge(u, v);
        dsu.save();
    }
    int q, online, tmax, last_ans = 0;
    cin >> q >> online >> tmax;
    while (q--) {
        int u, t;
        cin >> u >> t;
        u = (u + online * last_ans - 1) % n;
        t = (t + online * last_ans) % (tmax + 1);
        int tm = int(lower_bound(begin(ews), end(ews), t, greater<>()) -
                     ews.begin());
        dsu.rollback(tm);
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