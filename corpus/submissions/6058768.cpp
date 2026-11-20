#include "bits/extc++.h"

using namespace std;

#ifdef LOCAL
extern "C" {
void modify(int);
int query(int);
void report(int, int);
int check(int);

void explore(int, int);
}
#else
#include "explore.h"
#undef DEBUG
#endif

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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
ostream& operator<<(ostream& out, const vector<T>& arr) {
    out << "[";
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << ", ";
        }
        out << arr[i];
    }
    return out << "]";
}

template <typename A, typename T>
int lb(const A& arr, const T& x) {
    return int(lower_bound(begin(arr), end(arr), x) - arr.begin());
}

constexpr int maxn = 2e5 + 5;

struct G {
    bool arr[maxn] {};
    vector<int> graph[maxn];

    void modify(int u) {
        arr[u] ^= true;
        ::modify(u);
    }

    int query(int u) {
        bool ans = ::query(u) ^ arr[u];

        for (auto& v : graph[u]) {
            ans ^= arr[v];
        }

        return ans;
    }

    void report(int u, int v) {
        graph[u].push_back(v);
        graph[v].push_back(u);

        ::report(u, v);
    }

    bool check(int u) {
        return ::check(u);
    }
} G;

struct R {
    mt19937_64 cowng;

    // R() : cowng(chrono::steady_clock::now().time_since_epoch().count()) {}
    R() : cowng(1412) {}

    bool rbool() {
        return cowng() % 2;
    }

    template <typename T>
    void shuffle(T& arr) {
        ::shuffle(begin(arr), end(arr), cowng);
    }
} R;

template <typename Cb>
void do_bin_mapping(const vector<int>& base,
                    const vector<int>& queries,
                    const Cb& cb) {
    if (sz(base) == 1) {
        for (auto& a : queries) {
            cb(base[0], a);
        }

        return;
    } else if (!sz(queries)) {
        return;
    }

    int m = sz(base), mid = m / 2;

    for (int i = 0; i < mid; i++) {
        G.modify(base[i]);
    }

    vector<int> ch[2];

    for (auto& a : queries) {
        ch[G.query(a)].push_back(a);
    }

    for (int i = 0; i < mid; i++) {
        G.modify(base[i]);
    }

    do_bin_mapping(vector(base.begin(), base.begin() + mid), ch[1], cb);
    do_bin_mapping(vector(base.begin() + mid, base.end()), ch[0], cb);
}

vector<pair<int, int>> do_bin_mapping(const vector<int>& base,
                                      const vector<int>& queries) {
    vector<pair<int, int>> n_edges;

    do_bin_mapping(base, queries, [&](int u, int v) -> void {
        if (u > v) {
            swap(u, v);
        }

        n_edges.emplace_back(u, v);
    });

    sort(begin(n_edges), end(n_edges));
    n_edges.erase(unique(begin(n_edges), end(n_edges)), n_edges.end());

    return n_edges;
}

namespace s1 {

void solve(int n, int) {
    for (int i = 0; i + 1 < n; i++) {
        G.modify(i);

        for (int j = i + 1; j < n; j++) {
            if (G.query(j)) {
                G.report(i, j);
            }
        }
    }
}

}  // namespace s1

namespace s2 {

void solve(const vector<int>& pars, const vector<int>& chs) {
    dbg(pars, chs);
    if (sz(pars) == 1) {
        for (auto& a : chs) {
            G.report(pars[0], a);
        }

        return;
    } else if (!sz(chs)) {
        return;
    }

    int m = sz(pars), mid = m / 2;

    for (int i = 0; i < mid; i++) {
        G.modify(pars[i]);
    }

    vector<int> ch[2];
    for (auto& a : chs) {
        ch[a <= pars[mid - 1] || G.query(a)].push_back(a);
    }

    for (int i = 0; i < mid; i++) {
        G.modify(pars[i]);
    }

    solve(vector(pars.begin(), pars.begin() + mid), ch[1]);
    solve(vector(pars.begin() + mid, pars.end()), ch[0]);
}

void solve(int n, int) {
    vector<int> pars(n), chs(n - 1);
    iota(begin(pars), end(pars), 0);
    iota(begin(chs), end(chs), 1);

    solve(pars, chs);
}

}  // namespace s2

void explore(const int n, const int m_edges) {
    if (n <= 500) {
        s1::solve(n, m_edges);
        return;
    } else if (n % 10 == 7) {
        s2::solve(n, m_edges);
        return;
    }

    auto do_check = [&](int u) -> bool {
        if (n % 10 == 8) {
            return sz(G.graph[u]);
        }

        return G.check(u);
    };

    vector<int> nodes(n);
    iota(begin(nodes), end(nodes), 0);

    set<pair<int, int>> edges;

    dbg("A");
    while (sz(edges) < m_edges) {
        dbg(sz(edges), m_edges);
        int cn = sz(nodes);

        vector<int> base;

        for (auto& a : nodes) {
            if (R.rbool()) {
                base.push_back(a);
            }
        }

        for (auto& a : base) {
            G.modify(a);
        }

        vector<int> changed;

        for (auto& a : nodes) {
            if (G.query(a)) {
                changed.push_back(a);
            }
        }

        for (auto& a : base) {
            G.modify(a);
        }

        bool bad[cn] {};
        sort(begin(nodes), end(nodes));

        auto update_check = [&](int u) -> void {
            if (do_check(u)) {
                bad[lb(nodes, u)] = true;
            }
        };

        for (auto& [u, v] : do_bin_mapping(base, changed)) {
            if (!edges.emplace(u, v).second) {
                continue;
            }

            G.report(u, v);

            update_check(u);
            update_check(v);
        }

        vector<int> n_nodes;
        for (int i = 0; i < cn; i++) {
            if (!bad[i]) {
                n_nodes.push_back(nodes[i]);
            }
        }

        nodes = std::move(n_nodes);
    }
}