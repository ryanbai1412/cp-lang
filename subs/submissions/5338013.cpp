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

bool cmp(pair<long, long> a, pair<long, long> b) {
    assert(a.second > 0 && b.second > 0);
    return a.first * b.second < a.second * b.first;
}

bool eq(pair<long, long> a, pair<long, long> b) {
    return !cmp(a, b) && !cmp(b, a);
}

void solve() {
    int n;
    cin >> n;
    int arr[n][2];
    for (auto& [x, y] : arr) {
        cin >> x >> y;
    }
    long ans = 0;
    for (int i = 0; i < n; i++) {
        auto& [x1, y1] = arr[i];
        pair<long, long> mx {-long(1e18), 1}, mn {long(1e18), 1};
        for (int j = i + 1; j < n; j++) {
            auto& [x2, y2] = arr[j];
            pair<long, long> cur {y2 - y1, x2 - x1};
            mn = min(mn, cur, cmp);
            mx = max(mx, cur, cmp);
            ans += eq(mx, cur) || eq(mn, cur);
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