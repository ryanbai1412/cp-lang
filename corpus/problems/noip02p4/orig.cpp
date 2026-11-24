// Reference for noip02p4: accepted submission corpus/submissions/5096459.cpp, which covers
// k <= 3 by straight cuts (the judge's data never has k = 4), extended to the statement's k = 4:
// four disjoint rectangles split by a straight cut, or a pinwheel round an empty hole, tried at
// every hole between point coordinates in both turning directions with each point sent to its
// arm, O(n^5).
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


int solve4(const vector<pair<int, int>>& arr) {
    int ans = 1e9;
    split(arr, [&](const auto& a, const auto& b) -> void {
        ans = min(ans, solve1(a) + fs(b, solve3));
        ans = min(ans, fs(a, solve2) + fs(b, solve2));
        ans = min(ans, fs(a, solve3) + solve1(b));
    });
    return ans;
}

// arms round the hole x1 < x <= x2, y1 < y <= y2; -1 inside the hole
int arm(int x, int y, int x1, int x2, int y1, int y2) {
    if (y > y2) return x <= x2 ? 0 : 1;
    if (y > y1) return x > x2 ? 1 : x <= x1 ? 3 : -1;
    return x > x1 ? 2 : 3;
}

int pinwheel(const vector<pair<int, int>>& arr) {
    vector<int> xs, ys;
    for (auto& [x, y] : arr) xs.push_back(x), ys.push_back(y);
    sort(begin(xs), end(xs)), xs.erase(unique(begin(xs), end(xs)), end(xs));
    sort(begin(ys), end(ys)), ys.erase(unique(begin(ys), end(ys)), end(ys));
    int ans = 1e9;
    for (int x1 : xs) for (int x2 : xs) for (int y1 : ys) for (int y2 : ys) {
        if (x1 >= x2 || y1 >= y2) continue;
        int lx[4], hx[4], ly[4], hy[4];
        bool used[4] = {}, hole = false;
        for (auto& [x, y] : arr) {
            int a = arm(x, y, x1, x2, y1, y2);
            if (a < 0) { hole = true; break; }
            if (!used[a]) used[a] = true, lx[a] = hx[a] = x, ly[a] = hy[a] = y;
            lx[a] = min(lx[a], x), hx[a] = max(hx[a], x), ly[a] = min(ly[a], y), hy[a] = max(hy[a], y);
        }
        if (hole) continue;
        int s = 0;
        for (int a = 0; a < 4; a++) if (used[a]) s += (hx[a] - lx[a]) * (hy[a] - ly[a]);
        ans = min(ans, s);
    }
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
    } else if (kv == 3) {
        cout << fs(arr, solve3) << endl;
    } else {
        auto mirrored = arr;
        for (auto& [x, y] : mirrored) x = -x;
        cout << min({fs(arr, solve4), pinwheel(arr), pinwheel(mirrored)}) << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}