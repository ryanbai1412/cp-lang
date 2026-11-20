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
    constexpr mint(int x, bool) : x(x) {}

    friend ostream& operator<<(ostream& out, const mint& m) {
        return out << m.x;
    }

    mint operator-() const {
        if (!x) {
            return 0;
        }
        return mint(MOD - x, true);
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
        x = int(long(x) * m.x % MOD);
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

constexpr int maxn = 18;

int n;
vector<pair<int, int>> arr[maxn];
map<vector<int>, mint> memo[maxn];

mint dp(int ind, const vector<int>& comp) {
    if (ind == n - 1) {
        return 1;
    }
    auto [it, inserted] = memo[ind].emplace(comp, 0);
    if (!inserted) {
        return it->second;
    }
    mint& ans = it->second;
    for (auto& [u, v] : arr[ind]) {
        if (comp[u] == comp[v]) {
            continue;
        }
        auto tmp = comp;
        for (auto& a : tmp) {
            if (a == comp[u] || a == comp[v]) {
                a = min(comp[u], comp[v]);
            }
        }
        ans += dp(ind + 1, tmp);
    }
    return ans;
}

void solve() {
    cin >> n;
    for (int i = 0; i < n - 1; i++) {
        int m;
        cin >> m;
        arr[i].resize(m);
        for (auto& [u, v] : arr[i]) {
            cin >> u >> v;
            u--;
            v--;
        }
    }
    vector<int> comp(n);
    iota(begin(comp), end(comp), 0);
    cout << dp(0, comp) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}