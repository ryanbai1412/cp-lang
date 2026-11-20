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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

template <typename T>
istream& operator>>(istream& in, vector<T>& arr) {
    for (auto& a : arr) {
        in >> a;
    }
    return in;
}

template <typename T>
ostream& operator<<(ostream& out, const vector<T>& arr) {
    for (int i = 0; i < sz(arr); i++) {
        if (i) {
            out << " ";
        }
        out << arr[i];
    }
    return out;
}

template <typename T>
vector<T> dec(vector<T> arr) {
    for (auto& a : arr) {
        a--;
    }
    return arr;
}

template <typename T>
vector<T> inc(vector<T> arr) {
    for (auto& a : arr) {
        a++;
    }
    return arr;
}

template <typename T>
struct ST {
    int n;
    vector<vector<T>> v;

    ST() {}
    ST(const vector<T>& arr) : n(sz(arr)), v(__lg(n) + 2, vector<T>(n)) {
        v[0] = arr;
        for (int i = 1; i <= __lg(n) + 1; i++) {
            for (int j = 0; j + (1 << i) <= n; j++) {
                v[i][j] = min(v[i - 1][j], v[i - 1][j + (1 << (i - 1))]);
            }
        }
    }

    T query(int l, int r) const {
        int k = __lg(r - l + 1);
        return min(v[k][l], v[k][r - (1 << k) + 1]);
    }
};

struct Tree {
    int n;
    ST<pair<int, int>> st;
    vector<int> tin, tout, ord;
    vector<vector<int>> graph, lift;
    vector<pair<int, int>> ord2;

    Tree(vector<int> p)
        : n(sz(p)),
          tin(n),
          tout(n),
          graph(n),
          lift(__lg(n) + 2, vector<int>(n)) {
        for (int i = 0; i < n; i++) {
            if (p[i] >= 0) {
                graph[p[i]].push_back(i);
            }
        }
        for (int i = 0; i < n; i++) {
            if (p[i] < 0) {
                dfs(i, 0);
                p[i] = -1;
            }
        }
        st = ST(ord2);
        // lift[0] = p;
        // for (int i = 0; i <= __lg(n); i++) {
        //     for (int j = 0; j < n; j++) {
        //         if (lift[i][j] == -1) {
        //             lift[i + 1][j] = -1;
        //         } else {
        //             lift[i + 1][j] = lift[i][lift[i][j]];
        //         }
        //     }
        // }
    }

    void dfs(int u, int d) {
        tin[u] = sz(ord2);
        ord.push_back(u);
        for (auto& v : graph[u]) {
            ord2.emplace_back(d, u);
            dfs(v, d + 1);
        }
        ord2.emplace_back(d, u);
        tout[u] = sz(ord2) - 1;
    }

    bool anc(int u, int v) const {
        return tin[u] <= tin[v] && tout[v] <= tout[u];
    }

    int lca(int u, int v) const {
        // if (anc(u, v)) {
        //     return u;
        // }
        // for (int i = __lg(n); i >= 0; i--) {
        //     int x = lift[i][u];
        //     if (x != -1 && !anc(x, v)) {
        //         u = x;
        //     }
        // }
        // return lift[0][u];
        if (tin[u] > tin[v]) {
            swap(u, v);
        }
        return st.query(tin[u], tout[v]).second;
    }
};

vector<int> compute_psum(int n, const vector<array<int, 3>>& arr) {
    vector<int> graph[n];
    for (int i = 0; i < sz(arr); i++) {
        auto& [u, v, _] = arr[i];
        graph[u].push_back(i);
        graph[v].push_back(i);
    }
    bool vis[n] {};
    vector<int> q;
    int ans[n];
    memset(ans, -1, sizeof(ans));
    auto push = [&](int u, int x) -> void {
        if (!vis[u]) {
            vis[u] = true;
            ans[u] = x;
            q.push_back(u);
        } else {
            assert(ans[u] == x);
        }
    };
    push(arr[0][0], 0);
    while (sz(q)) {
        int cu = q.back();
        q.pop_back();
        for (auto& i : graph[cu]) {
            auto& [u, v, w] = arr[i];
            if (cu == u) {
                push(v, ans[u] - w);
            }
            if (cu == v) {
                push(u, ans[v] + w);
            }
        }
    }
    return vector<int>(ans, ans + n);
}

int solve(const Tree& t, const vector<int>& psum, int u, int v) {
    return psum[u] + psum[v] - 2 * psum[t.lca(u, v)];
}

void solve() {
    int n, m, q1, q2;
    cin >> n >> m >> q1 >> q2;
    vector<int> p1(n), p2(n);
    cin >> p1 >> p2;
    Tree t1(p1 = dec(p1)), t2(p2 = dec(p2));
    vector<int> nodes(t1.ord.begin(), t1.ord.begin() + m);
    cout << inc(nodes) << endl;
    sort(begin(nodes), end(nodes), [&](int a, int b) -> bool {
        return t2.tin[a] < t2.tin[b];
    });
    vector<array<int, 3>> w1, w2;
    for (int i = 0; i < m - 1; i++) {
        int u = nodes[i], v = nodes[i + 1];
        cout << "? " << u + 1 << " " << v + 1 << endl;
    }
    cout << "!" << endl;
    cout.flush();
    for (int i = 0; i < m - 1; i++) {
        int u = nodes[i], v = nodes[i + 1], w1u, w1v, w2u, w2v;
        cin >> w1u >> w1v >> w2u >> w2v;
        int l1 = t1.lca(u, v), l2 = t2.lca(u, v);
        w1.push_back({u, l1, w1u});
        w1.push_back({v, l1, w1v});
        w2.push_back({u, l2, w2u});
        w2.push_back({v, l2, w2v});
    }
    auto psum1 = compute_psum(n, w1), psum2 = compute_psum(n, w2);
    // dbg(psum1, psum2);
    // while (q2--) {
    //     int u, v;
    //     cin >> u >> v;
    //     u--;
    //     v--;
    //     cout << solve(t1, psum1, u, v) << " " << solve(t2, psum2, u, v) << endl;
    //     cout.flush();
    // }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}