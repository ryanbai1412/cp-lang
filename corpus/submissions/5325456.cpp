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

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

bool cmp(pair<int, int> a, pair<int, int> b) {
    return a.first + a.second < b.first + b.second;
}

void solve() {
    int n;
    cin >> n;
    int arr[2][n];
    for (auto& a : arr) {
        for (auto& b : a) {
            cin >> b;
        }
    }
    vector<pair<int, int>> vals;
    for (int i = 0; i < (1 << n); i++) {
        int sa = 0, sb = 0;
        for (int j = 0; j < n; j++) {
            if (on(i, j)) {
                sa += arr[0][j];
            } else {
                sb += arr[1][j];
            }
        }
        vals.emplace_back(sa, sb);
    }
    sort(begin(vals), end(vals), greater<>());
    pair<int, int> ans {INT_MAX, 0};
    for (int i = 0; i < sz(vals) - 1; i++) {
        vals[i + 1].second = max(vals[i + 1].second, vals[i].second);
        ans = min(ans, {vals[i].first + 1, vals[i + 1].second + 1}, cmp);
        ans = min(ans, {vals[i + 1].first + 1, vals[i].second + 1}, cmp);
    }
    cout << ans.first << endl;
    cout << ans.second << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}