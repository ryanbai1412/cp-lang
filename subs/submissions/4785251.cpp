#pragma GCC optimize("Ofast")
#pragma GCC target("avx2")

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
#define sz(x) int(size(x))

const int maxn = 1e7 + 5;

struct PFact {
    int mdiv[maxn + 1];

    PFact() : mdiv {} {
        for (int i = 2; i <= maxn; i++) {
            if (!mdiv[i]) {
                mdiv[i] = i;
                if (long(i) * i <= maxn) {
                    for (int j = i * i; j <= maxn; j += i) {
                        if (!mdiv[j]) {
                            mdiv[j] = i;
                        }
                    }
                }
            }
        }
    }

    vector<int> prime_factors(int x) const {
        vector<int> ans;
        while (x >= 2) {
            int p = mdiv[x];
            while (x % p == 0) {
                x /= p;
            }
            ans.push_back(p);
        }
        return ans;
    }
} pfact;

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

void solve() {
    int n, m, k;
    cin >> n >> m >> k;
    auto kfact = pfact.prime_factors(k);
    assert(sz(kfact) == 2);
    bool vis[2001] {};
    for (auto& a : kfact) {
        vis[a] = true;
    }
    long ans = 0, cache[n + 1];
    memset(cache, -1, sizeof(cache));
    for (int num = 1; num <= n; num++) {
        auto ifact = pfact.prime_factors(num);
        long key = 1;
        for (auto& a : ifact) {
            if (a > 2000 || !vis[a]) {
                key *= a;
            }
        }
        if (cache[key] != -1) {
            ans += cache[key];
            continue;
        }
        auto primes = kfact;
        for (auto& a : ifact) {
            if (a > 2000 || !vis[a]) {
                primes.push_back(a);
            }
        }
//        vector<int> primes(sz(kfact) + sz(ifact));
//        merge(begin(kfact), end(kfact), begin(ifact), end(ifact),
//              primes.begin());
//        primes.erase(unique(begin(primes), end(primes)), primes.end());
        long cans = 0;
        for (int i = 0; i < (1 << sz(primes)); i++) {
            long cur = 1;
#pragma GCC unroll 20
            for (int j = 0; j < sz(primes); j++) {
                if (on(i, j)) {
                    cur *= primes[j];
                }
            }
            long contrib = m / cur;
            if (__builtin_popcount(i) % 2 == 1) {
                cans -= contrib;
            } else {
                cans += contrib;
            }
        }
        ans += cans;
        cache[key] = cans;
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    solve();
}