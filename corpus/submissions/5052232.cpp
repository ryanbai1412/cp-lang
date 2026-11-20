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

struct ST1 {
    int n;
    vector<int> v;
    vector<multiset<int>> arr;

    ST1(int n = 0) : n(n), v(4 * n, 1e9), arr(n) {}

    void update(int o, int l, int r, int ind, int x) {
        if (l == r) {
            v[o] = x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            update(lc, l, mid, ind, x);
        } else {
            update(rc, mid + 1, r, ind, x);
        }
        v[o] = min(v[lc], v[rc]);
    }

    void update(int ind, int x) {
        update(1, 0, n - 1, ind, x);
    }
    
    int val(int ind) {
        if (!sz(arr[ind])) {
            return 1e9;
        }
        return *arr[ind].begin();
    }

    void insert(int ind, int x) {
        arr[ind].insert(x);
        update(ind, val(ind));
    }

    void remove(int ind, int x) {
        arr[ind].erase(x);
        update(ind, val(ind));
    }

    int query(int o, int l, int r, int ql, int qr) const {
        if (ql <= l && r <= qr) {
            return v[o];
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1, ans = 1e9;
        if (ql <= mid) {
            ans = min(ans, query(lc, l, mid, ql, qr));
        }
        if (mid < qr) {
            ans = min(ans, query(rc, mid + 1, r, ql, qr));
        }
        return ans;
    }

    int query(int l, int r) const {
        return query(1, 0, n - 1, l, r);
    }
    
    int query_suffix(int l) const {
        return query(l, n - 1);
    }
} st1;

struct DS {
    vector<int> arr;

    DS(const vector<int>& arr) : arr(arr) {}

    void update(int ind, int x) {
        arr[ind] = x;
    }

    int query(int l, int r, int x) const {
        int ans = 0;
        for (int i = l; i <= r; i++) {
            ans += arr[i] <= x;
        }
        return ans;
    }
};

constexpr int maxn = 2e5 + 5;

int tmc, tin[maxn], tout[maxn], depth[maxn];
vector<int> ord, graph[maxn], usets[maxn];
vector<pair<int, int>> upds[maxn];

void pdfs(int u, int p = -1, int d = 0) {
    for (auto& [kv, i] : upds[u]) {
        st1.insert(d + kv, i);
    }
    tin[u] = tmc++;
    depth[u] = d;
    int qi = st1.query_suffix(d);
    if (qi < 1e9) {
        dbg(qi, u);
        usets[qi].push_back(u);
    }
    for (auto& v : graph[u]) {
        if (v == p) {
            continue;
        }
        pdfs(v, u, d + 1);
    }
    for (auto& [kv, i] : upds[u]) {
        st1.remove(d + kv, i);
    }
    tout[u] = tmc;
}

void solve() {
    int n;
    cin >> n;
    for (int i = 0; i < n - 1; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    bool arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int q;
    cin >> q;
    int queries[q][3];
    for (auto& [t, u, kv] : queries) {
        cin >> t >> u >> kv;
        u--;
    }
    for (int i = 0; i < q; i++) {
        auto& [t, u, kv] = queries[i];
        if (t == 1) {
            upds[u].emplace_back(kv, i);
        }
    }
    st1 = ST1(n + 5);
    pdfs(0);
    vector<int> dsv(n, 1e9);
    for (int i = 0; i < n; i++) {
        if (arr[i]) {
            dsv[tin[i]] = depth[i];
        }
    }
    DS ds(dsv);
    for (int i = 0; i < q; i++) {
        auto& [t, u, kv] = queries[i];
        if (t == 1) {
            for (auto& a : usets[i]) {
                ds.update(tin[a], depth[a]);
            }
        } else {
            cout << ds.query(tin[u], tout[u] - 1, depth[u] + kv) << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}