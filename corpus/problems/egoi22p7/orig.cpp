// Accepted 5203797.cpp built with its LOCAL grader, which prints the query count and the design in place of a debug assert.
#define LOCAL
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

struct DSU {
    vector<int> p;

    DSU(int n) : p(n, -1) {}

    int find(int u) {
        return p[u] < 0 ? u : (p[u] = find(p[u]));
    }

    bool conn(int u, int v) {
        return find(u) == find(v);
    }

    void merge(int u, int v) {
        u = find(u);
        v = find(v);
        if (u == v) {
            return;
        }
        if (p[u] < p[v]) {
            swap(u, v);
        }
        p[v] += p[u];
        p[u] = v;
    }
};

struct Grader {
    int n, queries = 0;
    vector<DSU> arr;

    Grader() {}
    Grader(int n) : n(n) {
        int m;
        cin >> m;
        arr.emplace_back(n);
        for (int i = 0; i < m; i++) {
            int u, v;
            cin >> u >> v;
            u--;
            v--;
            arr[0].merge(u, v);
        }
    }

    int query_conn(int a, int u, int v) {
        queries++;
        u--;
        v--;
        if (arr[a].conn(u, v)) {
            return a;
        } else {
            arr.push_back(arr[a]);
            arr.back().merge(u, v);
            return sz(arr) - 1;
        }
    }

    void answer(vector<pair<int, int>> edges) {
        cout << queries << "\n" << sz(edges) << "\n";
        for (auto& [u, v] : edges) cout << u << " " << v << "\n";
        for (auto& [u, v] : edges) {
            u--;
            v--;
        }
        DSU dsu(n);
        for (auto& [u, v] : edges) {
            dsu.merge(u, v);
        }
        for (int i = 0; i < n; i++) {
            for (int j = 0; j < n; j++) {
                assert(dsu.conn(i, j) == arr[0].conn(i, j));
            }
        }
    }
} grader;

#ifdef LOCAL
int Connected(int a, int i, int j) {
    return grader.query_conn(a, i, j);
}

void DescribeDesign(vector<pair<int, int>> result) {
    return grader.answer(result);
}
#else
int Connected(int a, int i, int j);
void DescribeDesign(vector<pair<int, int>> result);
#endif

int query_conn(int a, int u, int v) {
    dbg("Q", a, u, v);
    u++;
    v++;
    return Connected(a, u, v);
}

void answer(vector<pair<int, int>> result) {
    for (auto& [u, v] : result) {
        dbg(u, v);
        u++;
        v++;
    }
    DescribeDesign(result);
}

constexpr int maxn = 205, tmaxn = maxn * 4;

DSU dsu(maxn);

int v[tmaxn], vrep[tmaxn];

template <typename Cb>
int bsearch(int o, int l, int r, int x, const Cb& cb) {
    if (l == r) {
        return l;
    }
    int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
    {
        for (int i = l; i <= mid; i++) {
            if (cb(i)) {
                goto loop1;
            }
        }
        return bsearch(rc, mid + 1, r, x, cb);
    loop1:;
    }
    {
        for (int i = mid + 1; i <= r; i++) {
            if (cb(i)) {
                goto loop2;
            }
        }
        return bsearch(lc, l, mid, x, cb);
    loop2:;
    }
    if (query_conn(v[lc], x, vrep[lc]) == v[lc]) {
        return bsearch(lc, l, mid, x, cb);
    } else {
        return bsearch(rc, mid + 1, r, x, cb);
    }
}

void build(int o, int l, int r) {
    if (l == r) {
        v[o] = 0;
        vrep[o] = l;
        return;
    }
    int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
    build(lc, l, mid);
    build(rc, mid + 1, r);
    vector<int> vis, visr;
    auto c_vis = [&](const vector<int>& arr, int u) -> bool {
        for (auto& a : arr) {
            if (dsu.conn(a, u)) {
                return true;
            }
        }
        return false;
    };
    v[o] = v[rc];
    for (int i = l; i <= mid; i++) {
        if (c_vis(vis, i)) {
            continue;
        }
        vis.push_back(i);
        int cv = query_conn(v[o], i, vrep[rc]);
        if (cv != v[o]) {
            v[o] = cv;
            continue;
        }
        int rv = bsearch(rc, mid + 1, r, i, [&](int u) -> bool {
            return !c_vis(visr, u);
        });
        visr.push_back(rv);
        dbg(i, rv);
        dsu.merge(i, rv);
    }
    vrep[o] = vrep[lc];
}

void ToyDesign(int n, int) {
    build(1, 0, n - 1);
    vector<pair<int, int>> ans;
    for (int i = 0; i < n; i++) {
        int p = dsu.find(i);
        if (p != i) {
            ans.emplace_back(p, i);
        }
    }
    answer(ans);
}

void solve() {
    int n;
    cin >> n;
    grader = Grader(n);
    ToyDesign(n, -1);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}