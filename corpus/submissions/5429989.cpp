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
    auto [sx, sy, sarr] = read();
    auto [tx, ty, tarr] = read();
    ld ans = 1e100;
    int si = 0, ti = 0;
    do {
        auto upd = [&](int& x, int& y, const pair<int, int>& p) -> void {
            auto& [dx, dy] = p;
            x += dx;
            y += dy;
        };
        ans = min(ans, hypotl(sx - tx, sy - ty));
        upd(sx, sy, sarr[si++]);
        upd(tx, ty, tarr[ti++]);
        if (si == sz(sarr)) {
            si = 0;
        }
        if (ti == sz(tarr)) {
            ti = 0;
        }
    } while (si || ti);
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(2);
    solve();
}