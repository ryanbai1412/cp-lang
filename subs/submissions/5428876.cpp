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

void solve() {
    auto read = [&](int& x, int& y) -> void {
        int d;
        char c;
        cin >> d >> c;
        if (c == 'X') {
            x = d;
            y = 0;
        } else {
            x = 0;
            y = d;
        }
    };
    int sx, sy, sn;
    cin >> sx >> sy >> sn;
    pair<int, int> arrs[sn];
    for (auto& [x, y] : arrs) {
        read(x, y);
    }
    int tx, ty, tn;
    cin >> tx >> ty >> tn;
    pair<int, int> arrt[tn];
    for (auto& [x, y] : arrt) {
        read(x, y);
    }
    auto upd = [&](int& x, int& y, const pair<int, int>& d) -> void {
        auto& [dx, dy] = d;
        x += dx;
        y += dy;
    };
    ld ans = 1e100;
    for (int i = 0; i <= 1e7; i++) {
        ans = min(ans, hypotl(sx - tx, sy - ty));
        upd(sx, sy, arrs[i % sn]);
        upd(tx, ty, arrt[i % tn]);
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(2);
    solve();
}