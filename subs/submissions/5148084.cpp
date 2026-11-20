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

template <typename T>
void merge(ost<T>*& a, ost<T>*& b) {
    if (sz(*a) < sz(*b)) {
        swap(a, b);
    }
    for (auto& v : *b) {
        a->insert(v);
    }
}

int query_lt(const ost<pair<int, int>>& s, int x) {
    return int(s.order_of_key({x, -1}));
}

int query_le(const ost<pair<int, int>>& s, int x) {
    return query_lt(s, x + 1);
}

int query_inclusive(const ost<pair<int, int>>& s, int l, int r) {
    return query_le(s, r) - query_lt(s, l);
}

ost<pair<int, int>> tost, cpar1, cpar2, heap[maxn];
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

ost<pair<int, int>>* dfs(int u, int p = -1) {
    ost<pair<int, int>>* ans = &heap[u];
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
        cpar1.insert({siz[v], v});
        cpar2.insert({n - siz[v], v});
        auto cur = dfs(v, u);
        cpar1.erase({siz[v], v});
        cpar2.erase({n - siz[v], v});
        go(siz[v], [&](int ql, int qr) -> int {
            return query_inclusive(*cur, ql, qr);
        });
        merge(ans, cur);
    }
    if (p != -1) {
        if (u == 1) {
            dbg(query_inclusive(tost, 2, 3), query_inclusive(*ans, 2, 3),
                query_inclusive(cpar1, 2, 3));
        }
        go(n - siz[u], [&](int ql, int qr) -> int {
            return query_inclusive(tost, ql, qr) -
                   query_inclusive(*ans, ql, qr) -
                   query_inclusive(cpar1, ql, qr) +
                   query_inclusive(cpar2, ql, qr);
        });
    }
    ans->insert({siz[u], u});
    return ans;
}

void solve() {
    cin >> n;
    tost.clear();
    for (int i = 0; i < n; i++) {
        heap[i].clear();
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
        tost.insert({siz[i], i});
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