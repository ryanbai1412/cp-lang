#pragma once
#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

using u32 = uint32_t;
using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

void init_io() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
}

mt19937_64 cowng(chrono::steady_clock::now().time_since_epoch().count());

long rint(long l, long r) {
    return cowng() % (r - l + 1) + l;
}

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

template <typename A, typename B>
ostream& operator<<(ostream& out, const pair<A, B>& p) {
    return out << "(" << p.first << ", " << p.second << ")";
}

vector<pair<int, int>> gen1(vector<int> gaps) {
    vector<pair<int, int>> v;

    int gap = 0;
    for (auto& nx : gaps) {
        if (gap == nx) {
            continue;
        } else if (gap < nx) {
            v.emplace_back(0, abs(nx - gap));
        } else {
            v.emplace_back(1, abs(nx - gap));
        }
    }

    return v;
}

vector<bool> gen_time(const vector<pair<int, int>>& strat) {
    vector<bool> arr;

    auto push = [&](int ty) -> void {
        if (sz(arr) && ty == 1 && !arr.back()) {
            arr.back() = true;
            return;
        }
        arr.push_back(!ty);
        arr.push_back(ty);
    };

    for (auto& [ty, cnt] : strat) {
        for (int i = 0; i < cnt; i++) {
            push(ty);
        }
    }

    return arr;
}

vector<vector<int>> to_adj_list(int n, const vector<pair<int, int>>& edges) {
    vector<vector<int>> graph(n);

    for (auto& [u, v] : edges) {
        graph[u].push_back(v);
        graph[v].push_back(u);
    }

    return graph;
}

struct Tree {
    int n;
    vector<vector<int>> graph;
    vector<int> par, depth;

    Tree(const vector<pair<int, int>>& edges, int rt)
        : n(sz(edges) + 1), graph(to_adj_list(n, edges)), par(n), depth(n) {
        dfs(rt, -1, 0);
    }

    void dfs(int u, int p, int d) {
        par[u] = p;
        depth[u] = d;

        for (auto& v : graph[u]) {
            if (v == p) {
                continue;
            }
            dfs(v, u, d + 1);
        }
    }

    int lca(int u, int v) const {
        while (depth[u] > depth[v]) {
            u = par[u];
        }
        while (depth[v] > depth[u]) {
            v = par[v];
        }
        while (u != v) {
            u = par[u];
            v = par[v];
        }
        return u;
    }

    int dist(int u, int v) const {
        return depth[u] + depth[v] - 2 * depth[lca(u, v)];
    }
};

void solve() {
    int n;
    cin >> n;
    if (n != 100) {
        return;
    }
    vector<pair<int, int>> edges(n - 1);
    for (auto& [u, v] : edges) {
        cin >> u >> v;
    }

    int rt = -1;
    for (int i = 0; i < n; i++) {
        auto cdist = Tree(edges, i).depth;
        if (*max_element(begin(cdist), end(cdist)) <= 50) {
            rt = i;
        }
    }

    assert(rt != -1);
    Tree tree(edges, rt);

    auto time_strat = gen_time(gen1({-3, 11, -22, 10, 25, -25, -30, 30}));

    cout << sz(time_strat) << endl;
    vector<vector<int>> ans[2];
    for (int ty : {0, 1}) {
        ans[ty].resize(n);

        for (int i = 0; i < n; i++) {
            int u = i;

            for (int j = 0; j < sz(time_strat); j++) {
                if (j % 2 == ty && time_strat[j] && u != rt) {
                    u = tree.par[u];
                }
                ans[ty][i].push_back(u);
                cout << u << " \n"[j == sz(time_strat) - 1];
            }
        }
    }

    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            if (i == j) {
                continue;
            }

            int d = tree.dist(i, j), t;
            for (t = 0; ans[0][i][t] != ans[1][j][t]; t++)
                ;
            dbg(double(t), d, i, j);
            assert(double(t) / d <= 13);
        }
    }
}

int main() {
    init_io();
    solve();
}