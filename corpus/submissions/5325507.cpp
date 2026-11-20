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

namespace s1 {

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

bool cmp(const pair<int, int>& a, const pair<int, int>& b) {
    return a.first + a.second < b.first + b.second;
}

pair<int, int> solve(const vector<int>& arra, const vector<int>& arrb) {
    int n = sz(arra);
    int sa = accumulate(begin(arra), end(arra), 0),
        sb = accumulate(begin(arrb), end(arrb), 0);
    pair<int, int> ans {INT_MAX, 0};
    for (int i = 0; i < (1 << n); i++) {
        int asum = 0, bmn = 1e9;
        for (int j = 0; j < n; j++) {
            if (on(i, j)) {
                asum += arra[j];
                bmn = min(bmn, arrb[j]);
            }
        }
        if (asum && bmn) {
            ans = min(ans, {sa - asum + 1, sb - bmn + 1}, cmp);
        }
    }
    return ans;
}

int solve1(const vector<int>& arra, const vector<int>& arrb) {
    auto [a, b] = min(solve(arra, arrb), solve(arrb, arra), cmp);
    return a + b;
}

}  // namespace s1

bool on(int mask, int bit) {
    return (mask >> bit) & 1;
}

bool cmp(pair<long, long> a, pair<long, long> b) {
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
    vector<pair<long, long>> vals;
    for (int i = 0; i < (1 << n); i++) {
        long sa = 0, sb = 0;
        for (int j = 0; j < n; j++) {
            if (on(i, j)) {
                sa += arr[0][j];
            } else {
                sb += arr[1][j];
            }
        }
        vals.emplace_back(sa, sb);
    }
    pair<long, long> ans {long(1e18), 0};
    int s0 = accumulate(arr[0], arr[0] + n, 0), s1 = accumulate(arr[1], arr[1] + n, 0);
    for (int x = 0; x <= s0; x++) {
        for (int y = 0; y <= s1; y++) {
            for (auto& [a, b] : vals) {
                if (x <= a && y <= b) {
                    goto loop;
                }
            }
            ans = min(ans, {x, y}, cmp);
        loop:;
        }
    }
    //    sort(begin(vals), end(vals));
    //    for (int i = sz(vals) - 1; i >= 1; i--) {
    //        vals[i - 1].second = max(vals[i - 1].second, vals[i].second);
    //    }
    //    for (int i = 0; i < sz(vals) - 1; i++) {
    //        ans = min(ans, {vals[i].first + 1, vals[i + 1].second + 1}, cmp);
    //    }
    //    ans = min(ans, {0, vals[0].second + 1}, cmp);
    //    ans = min(ans, {vals.back().first + 1, 0}, cmp);
    cout << ans.first << endl;
    cout << ans.second << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}