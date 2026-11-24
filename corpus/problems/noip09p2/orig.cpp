// corpus/submissions/5184950.cpp with each zero-initialised VLA cleared by memset: clang rejects `T a[n] {}`.
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

vector<int> primes;

void sieve(int n) {
    bool prime[n + 1];
    memset(prime, 0, sizeof prime);
    memset(prime, 1, sizeof(prime));
    for (int i = 2; i <= n; i++) {
        if (!prime[i]) {
            continue;
        }
        primes.push_back(i);
        if (i <= n / i) {
            for (int j = i * i; j <= n; j += i) {
                prime[j] = false;
            }
        }
    }
}

map<int, int> fact(int x) {
    map<int, int> ans;
    for (auto& a : primes) {
        int exp = 0;
        while (x % a == 0) {
            exp++;
            x /= a;
        }
        if (exp) {
            ans[a] = exp;
        }
    }
    if (x > 1) {
        ans[x] = 1;
    }
    return ans;
}

struct R {
    int mn = 0, mx = 1e9;

    R& operator+=(const R& r) {
        mn = max(mn, r.mn);
        mx = min(mx, r.mx);
        return *this;
    }

    int ways() const {
        return max(0, mx - mn + 1);
    }

    static R c_eq(int x) {
        return {x, x};
    }

    static R c_ge(int x) {
        return {x, int(1e9)};
    }

    static R c_le(int x) {
        return {0, x};
    }
};

void solve() {
    int a0, a1, b0, b1;
    cin >> a0 >> a1 >> b0 >> b1;
    map<int, R> tot;
    {
        auto f0 = fact(a0), f1 = fact(a1);
        for (auto& [p, e0] : f0) {
            int e1 = f1[p];
            if (e0 == e1) {
                tot[p] += R::c_ge(e1);
            } else {
                assert(e0 > e1);
                tot[p] += R::c_eq(e1);
            }
        }
    }
    {
        auto f0 = fact(b0), f1 = fact(b1);
        for (auto& [p, r] : tot) {
            if (b1 % p) {
                r += R::c_eq(0);
            }
        }
        for (auto& [p, e1] : f1) {
            int e0 = f0[p];
            if (e0 == e1) {
                tot[p] += R::c_le(e1);
            } else {
                assert(e0 < e1);
                tot[p] += R::c_eq(e1);
            }
        }
    }
    long ans = 1;
    for (auto& [p, r] : tot) {
        dbg(p, r.mn, r.mx);
        ans *= r.ways();
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    sieve(1e5);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}