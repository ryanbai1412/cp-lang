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

void solve() {
    int n;
    cin >> n;
    int arr[n][2];
    for (auto& [x, y] : arr) {
        cin >> x >> y;
    }
    for (auto& [x, y] : arr) {
        int nx = x + y, ny = y - x;
        x = nx;
        y = ny;
    }
    for (int i = 0; i < 2; i++) {
        vector<int> comp;
        for (auto& a : arr) {
            comp.push_back(a[i]);
        }
        sort(begin(comp), end(comp));
        comp.erase(unique(begin(comp), end(comp)), comp.end());
        for (auto& a : arr) {
            a[i] = int(lower_bound(begin(comp), end(comp), a[i]) - comp.begin());
        }
    }
    vector<int> xarr[n];
    for (auto& [x, y] : arr) {
        xarr[x].push_back(y);
    }
    int dp[n + 1];
    fill(dp, dp + n + 1, 1e9);
    dp[0] = -1;
    for (int i = n - 1; i >= 0; i--) {
        auto& a = xarr[i];
        sort(begin(a), end(a), greater<>());
        for (auto& b : a) {
            *lower_bound(dp, dp + n + 1, b) = b;
        }
    }
    cout << find(dp, dp + n + 1, 1e9) - dp - 1 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}