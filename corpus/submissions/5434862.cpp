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

long sq(long x) {
    return x * x;
}

void solve() {
    int n;
    cin >> n;
    array<int, 3> arr[n];
    for (auto& [x, y, r] : arr) {
        cin >> x >> y >> r;
    }
    sort(arr, arr + n);
    int cnt[n] {};
    for (auto& [xi, yi, ri] : arr) {
        for (auto it = lower_bound(arr, arr + n, array<int, 3> {xi - ri, -1, 0}); it != arr + n ; it++) {
            auto& [xj, yj, rj] = *it;
            if (xj > xi + ri) {
                break;
            } else if (ri <= rj) {
                continue;
            }
            cnt[it - arr] += sq(xi - xj) + sq(yi - yj) <= sq(ri);
        }
    }
    long ans = 0;
    for (int i = 0; i < n; i++) {
        long cx = sq(arr[i][2]);
        if (cnt[i] % 2 == 0) {
            ans += cx;
        } else {
            ans -= cx;
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