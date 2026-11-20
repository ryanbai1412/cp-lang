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

constexpr int maxa = 1e7 + 5;

int c_phi(int x) {
    static int cache[maxa + 1] {};
    if (cache[x]) {
        return cache[x];
    }
    int& ans = cache[x];
    ans = x;
    for (int i = 2; i <= int(1e5); i++) {
        if (x % i != 0) {
            continue;
        }
        ans = ans / i * (i - 1);
        while (x % i == 0) {
            x /= i;
        }
    }
    if (x > 1) {
        ans = ans / x * (x - 1);
    }
    return ans;
}

pair<bool, int> bpow(long base, long exp, int mod) {
    bool of = false, base_of = false;
    long ans = 1;
    while (exp) {
        if (exp & 1) {
            of |= ans * base >= mod || base_of;
            ans = (ans * base) % mod;
        }
        base_of |= base * base >= mod;
        base = (base * base) % mod;
        exp >>= 1;
    }
    return {of, ans};
}

int base;

pair<bool, int> cpow(int ind, int x, int mod) {
    if (!ind) {
        return {x >= mod, x % mod};
    }
    auto [of, nx] = cpow(ind - 1, x, c_phi(mod));
    if (of) {
        nx += c_phi(mod);
    }
    auto [cf, ans] = bpow(base, nx, mod);
    return {of || cf, ans};
}

void solve() {
    int n, q, mod;
    cin >> n >> q >> mod >> base;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int narr[n];
    memcpy(narr, arr, sizeof(arr));
    int cnt[n] {};
    while (q--) {
        int t, l, r;
        cin >> t >> l >> r;
        l--;
        r--;
        if (t == 0) {
            for (int i = l; i <= r; i++) {
                narr[i] = cpow(++cnt[i], arr[i], mod).second;
            }
        } else {
            int ans = 0;
            for (int i = l; i <= r; i++) {
                ans = (ans + narr[i]) % mod;
            }
            cout << ans << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}