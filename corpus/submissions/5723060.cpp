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

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

struct mint {
    static constexpr int MOD = 1e9 + 7;

    static constexpr int norm(long x) {
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
    }

    int x;

    constexpr mint() : x(0) {}
    constexpr mint(long x) : x(norm(x)) {}

    friend ostream& operator<<(ostream& out, const mint& m) {
        return out << m.x;
    }

    static constexpr mint from_unchecked(int x) {
        mint m;
        m.x = x;
        return m;
    }

    constexpr mint operator-() const {
        if (!x) {
            return {};
        }
        return mint::from_unchecked(MOD - x);
    }

    constexpr mint& operator+=(const mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    constexpr mint& operator-=(const mint& m) {
        return *this += -m;
    }

    constexpr mint& operator*=(const mint& m) {
        x = int(long(x) * m.x % MOD);
        return *this;
    }

    friend constexpr mint operator+(mint a, const mint& b) {
        return a += b;
    }
    friend constexpr mint operator-(mint a, const mint& b) {
        return a -= b;
    }
    friend constexpr mint operator*(mint a, const mint& b) {
        return a *= b;
    }
};

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

struct Tree {
    int n, cc = 0;
    vector<int> tin, tout;
    vector<vector<int>> graph;

    Tree(int n, int root, const vector<pair<int, int>>& edges)
        : n(n), tin(n), tout(n), graph(n) {
        for (auto& [u, v] : edges) {
            graph[u].push_back(v);
            graph[v].push_back(u);
        }
        dfs(root, -1);
    }

    void dfs(int u, int p) {
        tin[u] = cc++;

        for (auto& v : graph[u]) {
            if (v == p) {
                continue;
            }
            dfs(v, u);
        }

        tout[u] = cc;
    }

    bool anc(int u, int v) const {
        return tin[u] <= tin[v] && tout[v] <= tout[u];
    }
};

struct M {
    static constexpr int maxn = 2e5 + 5;

    mint pow2[maxn], i2 = (mint::MOD + 1) / 2;

    M() {
        pow2[0] = 1;
        for (int i = 1; i < maxn; i++) {
            pow2[i] = 2 * pow2[i - 1];
        }

        assert((2 * i2).x == 1);
    }
} M;

// struct DS {
//     int n;
//     vector<mint> mul, arr;

//     DS(int n) : n(n), mul(n, 1), arr(n, 0) {}

//     void update(int ind, mint x) {
//         arr[ind] = x;
//     }

//     void update_mul(int ind, mint x) {
//         for (int i = ind; i < n; i++) {
//             mul[i] *= x;
//         }
//     }

//     mint query_all() const {
//         mint ans = 0;
//         for (int i = 0; i < n; i++) {
//             ans += mul[i] * arr[i];
//         }
//         return ans;
//     }
// };

struct DS {
    int n;
    vector<mint> lazy, v;

    DS(int n) : n(n), lazy(4 * n, 1), v(4 * n, 0) {}

    void update(int o, int l, int r, int ind, mint x) {
        if (l == r) {
            v[o] = x * lazy[o];
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ind <= mid) {
            update(lc, l, mid, ind, x);
        } else {
            update(rc, mid + 1, r, ind, x);
        }
        v[o] = lazy[o] * (v[lc] + v[rc]);
    }

    void update(int ind, mint x) {
        update(1, 0, n - 1, ind, x);
    }

    void update_lazy(int o, int l, int r, int ql, int qr, mint x) {
        if (ql <= l && r <= qr) {
            v[o] *= x;
            lazy[o] *= x;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            update_lazy(lc, l, mid, ql, qr, x);
        }
        if (mid < qr) {
            update_lazy(rc, mid + 1, r, ql, qr, x);
        }
        v[o] = lazy[o] * (v[lc] + v[rc]);
    }

    void update_mul(int ind, mint x) {
        update_lazy(1, 0, n - 1, ind, n, x);
    }

    mint query_all() const {
        return v[1];
    }
};

mint solve(int m, vector<pair<int, int>> edges) {
    int n = sz(edges);

    sort(begin(edges), end(edges));

    vector<int> e_r[m], e_l[m];
    for (auto& [ql, qr] : edges) {
        e_r[qr].push_back(ql);
        e_l[ql].push_back(qr);
    }

    mint dp[m + 1];

    dp[m] = 1;

    mint no_r[m + 1], can_r[m + 1];

    fill(no_r, no_r + m + 1, 1);
    for (int j = 1; j <= m; j++) {
        can_r[j] = M.pow2[sz(e_r[j - 1])] - 1;
    }

    DS ds(m + 1);
    auto upd_ds = [&](int ind) -> void {
        ds.update(ind, can_r[ind] * dp[ind]);
    };

    upd_ds(m);

    for (int i = m - 1; i >= 0; i--) {
        dp[i] = ds.query_all();
        upd_ds(i);

        for (auto& qr : e_l[i]) {
            ds.update_mul(qr + 1, 2);
            can_r[qr + 1] = (can_r[qr + 1] + 1) * M.i2 - 1;
            upd_ds(qr + 1);
        }
    }

    return M.pow2[sz(edges)] - dp[0];

    // mint dp[n];

    // for (int i = n - 1; i >= 0; i--) {
    //     int c_cnt = 0;
    //     for (int j = i + 1; j < n; j++) {
    //         if (edges[i].second + 1 < edges[j].first) {
    //             break;
    //         }
    //         if (edges[j].second > edges[i].second) {
    //             dp[i] += M.pow2[c_cnt] * dp[j];
    //         }
    //         if (edges[j].second <= edges[i].second) {
    //             c_cnt++;
    //         }
    //     }

    //     if (edges[i].second == m - 1) {
    //         dp[i] += M.pow2[c_cnt];
    //     }
    // }

    // mint ans = M.pow2[sz(edges)];
    // for (int i = 0; i < n; i++) {
    //     if (!edges[i].first) {
    //         ans -= dp[i];
    //     }
    // }

    // return ans;

    // mint dp[m + 1] {};
    // dp[m] = 1;

    // for (int i = sz(edges) - 1; i >= 0; i--) {
    //     auto& [ql, qr] = edges[i];
    //     dbg(ql, qr);

    //     mint cdp[m + 1] {};
    //     copy(dp, dp + m + 1, cdp);

    //     for (int j = ql; j <= m; j++) {
    //         cdp[j] += dp[max(j, qr + 1)];
    //     }

    //     copy(cdp, cdp + m + 1, dp);
    // }

    // return M.pow2[sz(edges)] - dp[0];
}

void solve() {
    int tc_id, n, m, kv;
    cin >> tc_id >> n >> m >> kv;

    vector<pair<int, int>> tree(n - 1), edges(m);

    for (auto& [u, v] : tree) {
        cin >> u >> v;
        u--;
        v--;
    }

    for (auto& [u, v] : edges) {
        cin >> u >> v;
        u--;
        v--;
    }

    vector<int> arr(kv);
    for (auto& a : arr) {
        cin >> a;
        a--;
    }

    sort(begin(arr), end(arr));

    assert(arr[0] == 0);
    cout << M.pow2[m] << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}