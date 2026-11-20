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

constexpr long inv(long base, long mod) {
    return bpow(base, mod - 2, mod);
}

constexpr long mod = 1e9 + 7, base = 31, ibase = inv(base, mod);

struct PSA {
    vector<int> psum;

    PSA(int n) : psum(n + 1) {}

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
    vector<long> psum, ipows;

    Hasher(string s) : n(sz(s)) {
        psum.push_back(0);
        long pow = 1, ipow = 1;
        for (int i = 0; i < n; i++) {
            psum.push_back((psum.back() + (s[i] - 'a' + 1) * pow) % mod);
            ipows.push_back(ipow);
            pow = (pow * base) % mod;
            ipow = (ipow * ibase) % mod;
        }
    }

    long query(int l, int r) const {
        return (psum[r] - psum[l] + mod) * ipows[l] % mod;
    }

    int lcp(int l1, int l2) const {
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

    int lcs(int r1, int r2) const {
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
    PSA p_start(n), p_end(n);
    for (int bsize = 1; bsize <= n; bsize++) {
        auto upd = [&](int l, int r) -> void {
            if (l <= r) {
                dbg(l, r);
                p_start.update(l, r, 1);
                p_end.update(l + 2 * bsize - 1, r + 2 * bsize - 1, 1);
            }
        };
        for (int i = 0; i < n / bsize; i++) {
            int i1 = bsize * i, i2 = bsize * (i + 1), i3 = bsize * (i + 2);
            int lcs = min(bsize, h.lcs(i2, i3)),
                lcp = min(bsize, h.lcp(i2, i3));
            if (bsize == 3) {
                dbg(lcs, lcp, i1, i2, i3);
            }
            upd(max(i2 - lcs, i1), min(i2 - 1, i1 + lcp));
        }
    }
    long ans = 0;
    auto start = p_start.query(), end = p_end.query();
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