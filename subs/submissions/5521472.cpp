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

#define long __int128_t
#define int long

istream& operator>>(istream& in, long& x) {
    int64_t tmp;
    in >> tmp;
    x = tmp;
    return in;
}

ostream& operator<<(ostream& out, long x) {
    return out << int64_t(x);
}

int c_phi(int x) {
    int ans = x;
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

long bpow(long base, long exp, long mod) {
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

void solve() {
    int n, q, mod, base;
    cin >> n >> q >> mod >> base;
    long arr1[n], arr2[n];
    for (auto& a : arr1) {
        cin >> a;
    }
    memcpy(arr2, arr1, sizeof(arr2));
    long phi = c_phi(mod);
    int mreq = 0;
    {
        long cx = 1;
        while (cx < phi) {
            cx *= base;
            mreq++;
        }
    }
    while (q--) {
        int t, l, r;
        cin >> t >> l >> r;
        l--;
        r--;
        if (t == 0) {
            for (int i = l; i <= r; i++) {
                arr1[i] = bpow(base, arr2[i], mod);
                bool b = arr2[i] >= mreq;
                arr2[i] = bpow(base, arr2[i], phi);
                if (b) {
                    arr2[i] += phi;
                }
            }
        } else {
            long ans = 0;
            for (int i = l; i <= r; i++) {
                ans = (ans + arr1[i]) % mod;
            }
            cout << ans << endl;
        }
    }
}

int32_t main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}