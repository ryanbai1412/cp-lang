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

using u64 = uint64_t;

constexpr u64 base = 1e14 + 12;

void solve() {
    string s;
    cin >> s;
    int n = sz(s);
    u64 psum[n + 1], ipow[n + 1];
    ipow[0] = 1;
    for (int i = 0; i < n; i++) {
        psum[i + 1] = psum[i] * base + s[i];
        ipow[i + 1] = ipow[i] * base;
    }
    auto query = [&](int l, int r) -> u64 {
        return u64(psum[r] - psum[l] * ipow[r - l]);
    };
    int ans[n];
    iota(ans, ans + n, 0);
    sort(ans, ans + n, [&](int a, int b) -> bool {
        assert(a != b);
        int l = 0, r = n - max(a, b);
        while (l < r) {
            int mid = (l + r + 1) / 2;
            if (query(a, a + mid) == query(b, b + mid)) {
                l = mid;
            } else {
                r = mid - 1;
            }
        }
        if (a + l == n) {
            return true;
        } else if (b + l == n) {
            return false;
        }
        return s[a + l] < s[b + l];
    });
    for (int i = 0; i < n; i++) {
        cout << ans[i] << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}