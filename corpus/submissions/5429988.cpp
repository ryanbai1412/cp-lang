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

template <typename T>
vector<T> repeat(const vector<T>& arr, int kv) {
    vector<T> ans;
    for (int i = 0; i < kv; i++) {
        ans.insert(ans.end(), begin(arr), end(arr));
    }
    return ans;
}

template <typename T>
T sign(T x) {
    return (x > 0) - (x < 0);
}

tuple<int, int, vector<pair<int, int>>> read() {
    int sx, sy, n;
    cin >> sx >> sy >> n;
    vector<pair<int, int>> ans;
    for (int i = 0; i < n; i++) {
        int d, x, y;
        char c;
        cin >> d >> c;
        if (!d) {
            continue;
        }
        if (c == 'X') {
            x = sign(d);
            y = 0;
        } else {
            y = sign(d);
            x = 0;
        }
        for (int j = 0; j < abs(d); j++) {
            ans.emplace_back(x, y);
        }
    }
    return {sx, sy, ans};
}

void solve() {
    auto [sx, sy, arr] = read();
    {
        auto [tx, ty, arr2] = read();
        sx -= tx;
        sy -= ty;
        int tn = sz(arr2);
        arr = repeat(arr, tn);
        for (int i = 0; i < sz(arr); i++) {
            auto& [x1, y1] = arr[i];
            auto& [x2, y2] = arr2[i % tn];
            x1 -= x2;
            y1 -= y2;
        }
    }
    for (auto& [x, y] : arr) {
        dbg(x, y);
    }
    for (int i = 0; i < sz(arr) - 1; i++) {
        arr[i + 1].first += arr[i].first;
        arr[i + 1].second += arr[i].second;
    }
    auto& [dx, dy] = arr.back();
    dbg(dx, dy);
    auto sq = [&](ld x) -> ld { return x * x; };
    ld ans = sq(sx) + sq(sy);
    for (auto& [x, y] : arr) {
        int cx = sx + x, cy = sy + y;
        if (!dx && !dy) {
            ans = min(ans, sq(cx) + sq(cy));
            continue;
        }
        ld a = sq(dx) + sq(dy), b = long(2) * dx * cx + long(2) * dy * cy, c = sq(cx) + sq(cy);
        auto eval = [&](ld x) -> ld {
            return a * sq(x) + b * x + c;
        };
        ld ox = - b / (2 * a);
        ox = max(round(ox), ld(0));
        for (int i = -5; i <= 5; i++) {
            if (0 <= i + ox) {
                ans = min(ans, eval(i + ox));
            }
        }
        dbg(a, b, c, ans);
    }
    cout << sqrt(ans) << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(2);
    solve();
}