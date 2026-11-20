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

constexpr int maxn = 1e6 + 5;

struct M {
    mint fib[maxn + 1], fpsum[maxn + 2];

    M() {
        fib[0] = 0;
        fib[1] = 1;
        for (int i = 2; i <= maxn; i++) {
            fib[i] = fib[i - 1] + fib[i - 2];
        }
        fpsum[0] = fpsum[1] = 1;
        for (int i = 1; i <= maxn; i++) {
            fpsum[i + 1] = fpsum[i] * fib[i];
        }
    }

    mint query_fib(int l, int r) {
        return fpsum[r] / fpsum[l];
    }
} M;

struct PF {
    int pfact[maxn + 1];
    vector<int> divs[maxn + 1];

    PF() {
        iota(begin(pfact), end(pfact), 0);
        for (int i = 2; i <= maxn; i++) {
            if (pfact[i] != i || long(i) * i > maxn) {
                continue;
            }
            for (int j = i * i; j <= maxn; j += i) {
                if (pfact[j] == j) {
                    pfact[j] = i;
                }
            }
        }
        for (int i = 1; i <= maxn; i++) {
            divs[i] = query_divs(i);
        }
    }

    vector<int> query_divs(int n) {
        vector<int> ans;
        while (n > 1) {
            int x = pfact[n];
            if (x == -1) {
                ans.push_back(n);
                break;
            }
            ans.push_back(x);
            while (pfact[n] == x) {
                n /= x;
            }
        }
        return ans;
    }
} PF;

vector<pair<int, int>> solve1(int n) {
    vector<pair<int, int>> ans;
    for (int i = 1; i <= n;) {
        int k = n / i;
        ans.emplace_back(i, k);
        i = n / k + 1;
    }
    ans.emplace_back(n + 1, -1);
    return ans;
}

long solve2(int n, int m) {
    long ans = 0;
    for (int u = 1; u <= n; u++) {
        auto& d = PF.divs[u];
        for (int i = 0; i < (1 << sz(d)); i++) {
            int cmul = 1;
            for (int j = 0; j < sz(d); j++) {
                if (on(i, j)) {
                    cmul *= d[j];
                }
            }
            if (__builtin_popcount(i) % 2 == 0) {
                ans += m / cmul;
            } else {
                ans -= m / cmul;
            }
        }
    }
    return ans;
}

void solve() {
    int n, m;
    cin >> n >> m;
    auto an = solve1(n), am = solve1(m);
    mint ans = 1;
    int ci = 1, nv = -1, mv = -1, ni = 0, mi = 0;
    while (ni < sz(an) && mi < sz(am)) {
        int nxtt = min(an[ni].first, am[mi].first);
        if (ci < nxtt) {
            dbg(ci, nxtt, M.query_fib(ci, nxtt), nv, mv);
            ans *= mint::pow(M.query_fib(ci, nxtt), solve2(nv, mv));
        }
        if (an[ni].first == nxtt) {
            nv = an[ni++].second;
        }
        if (am[mi].first == nxtt) {
            mv = am[mi++].second;
        }
        ci = nxtt;
    }
    cout << ans << endl;
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