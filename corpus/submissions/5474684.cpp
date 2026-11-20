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

struct mint {
    static constexpr int MOD = 10007;

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

    mint operator-() const {
        if (!x) {
            return 0;
        }
        return mint::from_unchecked(MOD - x);
    }

    mint& operator+=(const mint& m) {
        x += m.x;
        if (x >= MOD) {
            x -= MOD;
        }
        return *this;
    }
    mint& operator-=(const mint& m) {
        return *this += -m;
    }

    mint& operator*=(const mint& m) {
        x = int(x * m.x % MOD);
        return *this;
    }

    friend mint operator+(mint a, const mint& b) {
        return a += b;
    }
    friend mint operator-(mint a, const mint& b) {
        return a -= b;
    }
    friend mint operator*(mint a, const mint& b) {
        return a *= b;
    }
};

vector<mint> mul(const vector<mint>& arra, const vector<mint>& arrb) {
    int n = sz(arra);
    vector<mint> ans(n);
    for (int i = 0; i < n; i++) {
        for (int j = 0; j < n; j++) {
            ans[i ^ j] += arra[i] * arrb[j];
        }
    }
    return ans;
}

vector<mint> add(vector<mint> arra, const vector<mint>& arrb) {
    for (int i = 0; i < sz(arra); i++) {
        arra[i] += arrb[i];
    }
    return arra;
}

vector<mint> cxor(const vector<mint>& arr, int x) {
    int n = sz(arr);
    vector<mint> ans(n);
    for (int i = 0; i < n; i++) {
        ans[i ^ x] = arr[i];
    }
    return ans;
}

namespace s1 {

constexpr int maxn = 2e3 + 5;

int m;
vector<int> arr, graph[maxn];
vector<mint> ans;

vector<mint> dfs(int u, int p) {
    vector<mint> cans(m);
    cans[arr[u]] += 1;
    for (auto& v : graph[u]) {
        if (v == p) {
            continue;
        }
        cans = mul(cans, dfs(v, u));
    }
    ans = add(ans, cans);
    cans[0] += 1;
    return cans;
}

bool solve(int _m,
           const vector<int>& _arr,
           const vector<pair<int, int>>& edges,
           const vector<pair<int, int>>& queries) {
    m = _m;
    arr = _arr;
    for (auto& a : queries) {
        if (a.first != -1) {
            return false;
        }
    }
    ans.resize(m);
    for (auto& [u, v] : edges) {
        graph[u].push_back(v);
        graph[v].push_back(u);
    }
    dfs(0, -1);
    for (auto& [_t, u] : queries) {
        cout << ans[u] << endl;
    }
    return true;
}
}  // namespace s1

namespace s2 {

int m;

struct ST {
    int n;
    vector<int> lazy, arr;
    vector<vector<mint>> v;

    ST(const vector<int>& arr) : n(sz(arr)), lazy(4 * n), arr(arr), v(4 * n) {
        build(1, 0, n - 1);
    }

    void build(int o, int l, int r) {
        if (l == r) {
            v[o] = vector<mint>(m);
            v[o][arr[l]] = 1;
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        build(lc, l, mid);
        build(rc, mid + 1, r);
        v[o] = add(v[lc], v[rc]);
    }

    void update(int o, int l, int r, int ql, int qr, int x) {
        if (ql <= l && r <= qr) {
            lazy[o] ^= x;
            v[o] = cxor(v[o], x);
            return;
        }
        int mid = (l + r) / 2, lc = o * 2, rc = lc + 1;
        if (ql <= mid) {
            update(lc, l, mid, ql, qr, x);
        }
        if (mid < qr) {
            update(rc, mid + 1, r, ql, qr, x);
        }
        v[o] = cxor(add(v[lc], v[rc]), x);
    }

    void update(int l, int r, int x) {
        update(1, 0, n - 1, l, r, x);
    }

    vector<mint> query_all() const {
        return v[1];
    }
};

bool solve(int _m,
           const vector<int>& arr,
           const vector<pair<int, int>>& edges,
           const vector<pair<int, int>>& queries) {
    m = _m;
    int n = sz(arr);
    for (int i = 0; i < n - 1; i++) {
        auto& [u, v] = edges[i];
        if (!(i == u && i + 1 == v)) {
            return false;
        }
    }
    auto starr = arr;
    starr.insert(starr.begin(), 0);
    for (int i = 0; i < n; i++) {
        starr[i + 1] ^= starr[i];
    }
    ST st(starr);
    auto c2 = [&](int x) -> mint { return mint(x * (x - 1) / 2); };
    for (auto& [u, x] : queries) {
        if (u == -1) {
            auto cx = st.query_all();
            mint ans = 0;
            if (!x) {
                for (auto& a : cx) {
                    ans += c2(a.x);
                }
            } else {
                for (int i = 0; i < m; i++) {
                    if ((i ^ x) > i) {
                        ans += cx[i] * cx[i ^ x];
                    }
                }
            }
            cout << ans << endl;
        } else {
            st.update(u + 1, n, x);
        }
    }
    return true;
}
}  // namespace s2

void solve() {
    int n, m;
    cin >> n >> m;
    m = 1 << (__lg(m - 1) + 1);
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }
    vector<pair<int, int>> edges;
    for (int i = 0; i < n - 1; i++) {
        int u, v;
        cin >> u >> v;
        u--;
        v--;
        edges.emplace_back(u, v);
    }
    int q;
    cin >> q;
    vector<pair<int, int>> queries(q);
    for (auto& [u, x] : queries) {
        string s;
        cin >> s;
        if (s == "Change") {
            cin >> u >> x;
            u--;
        } else {
            assert(s == "Query");
            cin >> x;
            u = -1;
        }
    }
    s1::solve(m, arr, edges, queries) || s2::solve(m, arr, edges, queries);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}