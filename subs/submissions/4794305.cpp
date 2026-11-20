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

constexpr long base = 31, mod = 1e9 + 7, ibase = inv(base, mod);

struct PSA {
    int n;
    vector<int> psum;

    PSA(int n) : n(n), psum(n + 1) {}

    void update(int l, int r, int x) {
        psum[l] += x;
        psum[r + 1] -= x;
    }

    vector<int> query() const {
        vector<int> ans(sz(psum));
        partial_sum(begin(psum), end(psum), ans.begin());
        return ans;
    }
};

struct Hasher {
    int n;
    vector<long> psum, ipow;

    Hasher(string s) : n(sz(s)) {
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

    int lcs(int l1, int l2) const {
        if (l1 >= n || l2 >= n) {
            return 0;
        }
        int l = 0, r = n - max(l1, l2);
        while (l < r) {
            int mid = (l + r + 1) / 2;
            if (query(l1, l1 + mid) == query(l2, l2 + mid)) {
                l = mid;
            } else {
                r = mid - 1;
            }
        }
        return l;
    }

    int lcp(int r1, int r2) const {
        if (r1 > n || r2 > n) {
            return 0;
        }
        int l = 0, r = min(r1, r2);
        while (l < r) {
            int mid = (l + r + 1) / 2;
            if (query(r1 - mid, r1) == query(r2 - mid, r2)) {
                l = mid;
            } else {
                r = mid - 1;
            }
        }
        return l;
    }
};

void solve() {
    string s;
    cin >> s;
    int n = sz(s);
    Hasher h(s);
    PSA pstart(n), pend(n);
    for (int i = 1; i <= n; i++) {
        auto upd = [&](int l, int r) -> void {
            if (l <= r) {
                dbg(l, r);
                pstart.update(l, r, 1);
                pend.update(l + 2 * i - 1, r + 2 * i - 1, 1);
            }
        };
        for (int j = 0; j < n / i; j++) {
            int lcs = min(i, h.lcs((j + 1) * i, (j + 2) * i)),
                lcp = min(i, h.lcp((j + 1) * i, (j + 2) * i));
            dbg(lcs, lcp);
            upd(max((j + 1) * i - lcp, j * i),
                min((j + 1) * i - 1, j * i + lcs));
        }
    }
    auto start = pstart.query(), end = pend.query();
    long ans = 0;
    for (int i = 0; i < n - 1; i++) {
        ans += end[i] * start[i + 1];
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