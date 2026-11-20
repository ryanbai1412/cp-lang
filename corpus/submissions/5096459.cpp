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

template <typename T>
T swapped(T arr) {
    for (auto& [a, b] : arr) {
        swap(a, b);
    }
    return arr;
}

template <typename T, typename Cb>
int fs(const T& arr, const Cb& cb) {
    return min(cb(arr), cb(swapped(arr)));
}

template <typename Cb>
void split(vector<pair<int, int>> arr, const Cb& cb) {
    sort(begin(arr), end(arr));
    int prv = -1;
    for (int i = 0; i < sz(arr); i++) {
        if (prv < arr[i].first) {
            cb(vector<pair<int, int>>(arr.begin(), arr.begin() + i),
               vector<pair<int, int>>(arr.begin() + i, arr.end()));
        }
        prv = arr[i].first;
    }
}

int solve1(const vector<pair<int, int>>& arr) {
    if (!sz(arr)) {
        return 0;
    }
    int xl = 1e9, xr = -1e9, yl = 1e9, yr = -1e9;
    for (auto& [x, y] : arr) {
        xl = min(xl, x);
        xr = max(xr, x);
        yl = min(yl, y);
        yr = max(yr, y);
    }
    return (xr - xl) * (yr - yl);
}

int solve2(const vector<pair<int, int>>& arr) {
    int ans = 1e9;
    split(arr, [&](const auto& a, const auto& b) -> void {
        ans = min(ans, solve1(a) + solve1(b));
    });
    return ans;
}

int solve3(const vector<pair<int, int>>& arr) {
    int ans = 1e9;
    split(arr, [&](const auto& a, const auto& b) -> void {
        ans = min(ans, solve1(a) + fs(b, solve2));
        ans = min(ans, fs(a, solve2) + solve1(b));
    });
    return ans;
}

void solve() {
    int n, kv;
    cin >> n >> kv;
    vector<pair<int, int>> arr(n);
    for (auto& [x, y] : arr) {
        cin >> x >>y;
    }
    if (kv == 1) {
        cout << solve1(arr) << endl;
    } else if (kv == 2) {
        cout << fs(arr, solve2) << endl;
    } else {
        cout << fs(arr, solve3) << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}