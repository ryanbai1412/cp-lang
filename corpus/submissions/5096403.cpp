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

using ld = long double;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr ld eps = 1e-5;

void solve() {
    ld h, s1, v, l, k;
    int n;
    cin >> h >> s1 >> v >> l >> k >> n;
    k += eps;
    auto gy = [&](ld x) -> ld {
        x = min(x, s1);
        ld t = (s1 - x) / v;
        return h - 5 * t * t;
    };
    int ans = 0;
    for (int i = 0; i < n; i++) {
        ld yl = gy(i - l - eps), yr = gy(i + eps);
        if ((yl <= 0 && 0 <= yr) || (yl <= k && k <= yr) ||
            (0 <= yl && k <= yr)) {
            dbg(i, yl, yr);
            ans++;
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}