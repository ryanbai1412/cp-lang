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

namespace __gnu_pbds {
template <typename T, typename U = less<T>>
using ost =
    tree<T, null_type, U, rb_tree_tag, tree_order_statistics_node_update>;
}

using __gnu_pbds::ost;

struct DS1 {
    vector<int> v;

    DS1() : v {0} {}

    void insert(int x) {
        v.push_back(x);
    }

    void build() {
        sort(begin(v), end(v), greater<>());
    }

    int mx_excluding(int x) {
        if (v[0] == x) {
            return v[1];
        }
        return v[0];
    }
};

constexpr int maxn = 3e5 + 5;

struct Node {
    int v, lc, rc;
} heap[1 << 23];

int heap_ptr;

int alloc() {
    heap[heap_ptr] = {};
    return heap_ptr++;
}

void update(int& o, int l, int r, int ind, int x) {
    if (!o) {
        o = alloc();
    }
    auto& [v, lc, rc] = heap[o];
    if (l == r) {
        v += x;
        return;
    }
    int mid = (l + r) / 2;
    if (ind <= mid) {
        update(lc, l, mid, ind, x);
    } else {
        update(rc, mid + 1, r, ind, x);
    }
    v = heap[lc].v + heap[rc].v;
}

void update(int& o, int ind, int x) {
    update(o, 0, maxn, ind, x);
}

int query(int o, int l, int r, int ql, int qr) {
    if (!o) {
        return 0;
    }
    auto& [v, lc, rc] = heap[o];
    if (ql <= l && r <= qr) {
        return v;
    }
    int mid = (l + r) / 2, ans = 0;
    if (ql <= mid) {
        ans += query(lc, l, mid, ql, qr);
    }
    if (mid < qr) {
        ans += query(rc, mid + 1, r, ql, qr);
    }
    return ans;
}

int query(int o, int l, int r) {
    return query(o, 0, maxn, l, r);
}

void merge(int& a, int b, int l, int r) {
    if (!a) {
        a = b;
        return;
    }
    if (!b) {
        return;
    }
    auto& [v, lc, rc] = heap[a];
    if (l == r) {
        v += heap[b].v;
        return;
    }
    int mid = (l + r) / 2;
    merge(lc, heap[b].lc, l, mid);
    merge(rc, heap[b].rc, mid + 1, r);
    v = heap[lc].v + heap[rc].v;
}

void merge(int& a, int& b) {
    if (heap[a].v < heap[b].v) {
        swap(a, b);
    }
    merge(a, b, 0, maxn);
}

int query_inclusive(int s, int l, int r) {
    return query(s, l, r);
}

int tost, cpar1, cpar2;
vector<int> graph[maxn];
int n, siz[maxn];
long tans;

void pdfs(int u, int p = -1) {
    siz[u] = 1;
    for (auto& v : graph[u]) {
        if (v == p) {
            continue;
        }
        pdfs(v, u);
        siz[u] += siz[v];
    }
}

int dfs(int u, int p = -1) {
    int ans = 0;
    DS1 ssz;
    for (auto& v : graph[u]) {
        if (v == p) {
            continue;
        }
        ssz.insert(siz[v]);
    }
    if (p != -1) {
        ssz.insert(n - siz[u]);
    }
    ssz.build();
    auto go = [&](int csz, auto cb) -> void {
        int mxsz = ssz.mx_excluding(csz);
        int cmn = 2 * mxsz - n + csz, cmx = n - csz;
        int cways = cb(csz - cmx, csz - cmn);
        dbg(u, cways, cmn, cmx);
        tans += long(u + 1) * cways;
    };
    for (auto& v : graph[u]) {
        if (v == p) {
            continue;
        }
        update(cpar1, siz[v], 1);
        update(cpar2, n - siz[v], 1);
        auto cur = dfs(v, u);
        update(cpar1, siz[v], -1);
        update(cpar2, n - siz[v], -1);
        go(siz[v], [&](int ql, int qr) -> int {
            return query_inclusive(cur, ql, qr);
        });
        merge(ans, cur);
    }
    if (p != -1) {
        go(n - siz[u], [&](int ql, int qr) -> int {
            return query_inclusive(tost, ql, qr) -
                   query_inclusive(ans, ql, qr) -
                   query_inclusive(cpar1, ql, qr) +
                   query_inclusive(cpar2, ql, qr);
        });
    }
    update(ans, siz[u], 1);
    return ans;
}

void solve() {
    cin >> n;
    heap_ptr = 1;
    tost = cpar1 = cpar2 = 0;
    for (int i = 0; i < n; i++) {
        graph[i].clear();
    }
    tans = 0;
    for (int i = 0; i < n - 1; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    pdfs(0);
    for (int i = 1; i < n; i++) {
        update(tost, siz[i], 1);
    }
    dfs(0);
    cout << tans << endl;
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