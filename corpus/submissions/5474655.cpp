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
    s1::solve(m, arr, edges, queries);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}