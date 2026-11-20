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

void solve() {
    auto read = [&](int& x, int& y) -> void {
        int d;
        char c;
        cin >> d >> c;
        if (c == 'X') {
            x = d;
            y = 0;
        } else {
            y = d;
            x = 0;
        }
    };
    int sx, sy, sn;
    cin >> sx >> sy >> sn;
    vector<pair<int, int>> arr(sn);
    for (auto& [x, y] : arr) {
        read(x, y);
    }
    {
        int tx, ty, tn;
        cin >> tx >> ty >> tn;
        sx -= tx;
        sy -= ty;
        pair<int, int> arr2[tn];
        for (auto& [x, y] : arr2) {
            read(x, y);
        }
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
    auto sq = [&](ld x) -> ld { return x * x; };
    ld ans = sq(sx) + sq(sy);
    for (auto& [x, y] : arr) {
        int cx = sx + x, cy = sy + y;
        ld a = sq(dx) + sq(dy), b = long(2) * dx * cx + long(2) * dy * cy, c = sq(cx) + sq(cy);
        auto eval = [&](ld x) -> ld {
            return a * sq(x) + b * x + c;
        };
        ld ox = - b / (2 * a);
        ox = max(round(ox), ld(1));
        for (int i = -5; i <= 5; i++) {
            if (1 <= i + ox) {
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