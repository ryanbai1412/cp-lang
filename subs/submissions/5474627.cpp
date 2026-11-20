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
    int n, kv;
    cin >> n >> kv;
    int arr[n][3];
    for (auto& [p, cap, w] : arr) {
        cin >> p >> cap >> w;
    }
    long ans1 = min(arr[0][1], kv + 1) * long(arr[0][2]);
    long ans = 0;
    vector<pair<int, int>> ord;
    for (int i = 1; i < n; i++) {
        ord.emplace_back(arr[i][2], arr[i][1]);
    }
    sort(begin(ord), end(ord), greater<>());
    ans += ord[0].first;
    ord[0].second--;
    ans += arr[0][2];
    ord.emplace_back(arr[0][2], arr[0][1] - 1);
    sort(begin(ord), end(ord), greater<>());
    for (auto& [w, cap] : ord) {
        int sub = min(kv, cap);
        kv -= sub;
        ans += long(sub) * w;
    }
    cout << max(ans1, ans) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    int tcs;
    cin >> tcs;
    while (tcs--) {
        solve();
    }
}