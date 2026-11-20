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

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

struct mint {
    static constexpr int MOD = 1e9 + 7;

    static constexpr int norm(long x) {
        x %= MOD;
        if (x < 0) {
            x += MOD;
        }
        return int(x);
    }

    static constexpr mint pow(mint base, long exp) {
        mint ans = 1;
        while (exp) {
            if (exp & 1) {
                ans *= base;
            }
            base *= base;
            exp >>= 1;
        }
        return ans;
    }

    static constexpr mint inv(mint x) {
        return pow(x, MOD - 2);
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
    mint& operator/=(const mint& m) {
        return *this *= mint::inv(m);
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
    friend mint operator/(mint a, const mint& b) {
        return a /= b;
    }
};

mint det(vector<vector<mint>> arr) {
    int n = sz(arr);
    mint ans = 1;

    for (int i = 0; i < n; i++) {
        for (int j = i; j < n; j++) {
            if (arr[j][i].x) {
                swap(arr[i], arr[j]);
                break;
            }
        }

        if (!arr[i][i].x) {
            continue;
        }

        mint i_div = 1 / arr[i][i];

        for (int j = i + 1; j < n; j++) {
            mint cmul = arr[j][i] * i_div;

            for (int k = i; k < n; k++) {
                arr[j][k] -= cmul * arr[i][k];
            }
        }
    }

    for (int i = 0; i < n; i++) {
        ans *= arr[i][i];
    }

    return ans;
}

mint count_trees(int n, const vector<pair<int, int>>& edges) {
    int deg[n] {};

    vector<vector<mint>> mat(n, vector<mint>(n));

    for (auto& [u, v] : edges) {
        if (u == v) {
            continue;
        }

        mat[u][v] -= 1;
        mat[v][u] -= 1;
        deg[u]++;
        deg[v]++;
    }

    for (int i = 0; i < n; i++) {
        mat[i][i] = deg[i];
    }

    mat.pop_back();
    for (auto& a : mat) {
        a.pop_back();
    }

    return det(mat);
}

void solve() {
    int n;
    cin >> n;

    vector<pair<int, int>> edges[n - 1];
    for (auto& e : edges) {
        int m;
        cin >> m;
        e.resize(m);

        for (auto& [u, v] : e) {
            cin >> u >> v;
            u--;
            v--;
        }
    }

    auto pow_n1 = [&](int x) -> mint {
        if (x & 1) {
            return -1;
        } else {
            return 1;
        }
    };

    mint ans = 0;

    for (int mask = 0; mask < (1 << (n - 1)); mask++) {
        vector<pair<int, int>> c_edges;

        for (int i = 0; i < n - 1; i++) {
            if (on(mask, i)) {
                continue;
            }

            c_edges.insert(c_edges.end(), begin(edges[i]), end(edges[i]));
        }

        ans += pow_n1(__builtin_popcount(mask)) * count_trees(n, c_edges);
    }

    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}