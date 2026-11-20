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

constexpr long bpow(long base, long exp, long mod) {
    long ans = 1;
    while (exp) {
        if (exp & 1) {
            ans = (ans * base) % mod;
        }
        base = (base * base) % mod;
        exp >>= 1;
    }
    return ans;
}

constexpr long inv(long x, long mod) {
    return bpow(x, mod - 2, mod);
}

const long base = 31, mod = 1e9 + 7, ibase = inv(base, mod);

struct Hasher {
    vector<long> psum, ipow;

    Hasher(string s) {
        psum.push_back(0);
        long cpow = 1, cipow = 1;
        for (int i = 0; i < sz(s); i++) {
            psum.push_back((psum.back() + cpow * (s[i] - 'a' + 1)) % mod);
            ipow.push_back(cipow);
            cpow = (cpow * base) % mod;
            cipow = (cipow * ibase) % mod;
        }
    }

    long query(int l, int r) const {
        return (psum[r] - psum[l] + mod) * ipow[l] % mod;
    }
};

void solve() {
    string s;
    cin >> s;
    int n = sz(s);
    Hasher h(s);
    long starts[n] {}, ends[n] {};
    for (int i = 0; i < n; i++) {
        for (int j = 1; i + 2 * j <= n; j++) {
            if (h.query(i, i + j) == h.query(i + j, i + 2 * j)) {
                starts[i]++;
                ends[i + 2 * j - 1]++;
            }
        }
    }
    long ans = 0;
    for (int i = 0; i < n - 1; i++) {
        ans += ends[i] * starts[i + 1];
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    ios_base::sync_with_stdio(false);
    cin.exceptions(ios::failbit);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}