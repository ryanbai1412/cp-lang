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

long solve(const vector<pair<int, int>>& arr) {
    long ans = 1e18;
    for (int xl = 0; xl < 100; xl++) {
        for (int xr = 0; xr < 100; xr++) {
            long cans = 0;
            for (auto& [a, b] : arr) {
                cans += min(abs(xl - a) + b, abs(xr - a) + 1 - b);
            }
            ans = min(ans, cans);
        }
    }
    return ans;
}

void solve() {
    int n;
    cin >> n;
    int arr[n][3];
    for (auto& [x, y, v] : arr) {
        cin >> x >> y >> v;
    }
    long ans = 0;
    {
        vector<pair<int, int>> vx;
        for (auto& [x, _y, v] : arr) {
            if (v == 0 || v == 1) {
                vx.emplace_back(x, 0);
            } else {
                vx.emplace_back(x, 1);
            }
        }
        ans += solve(vx);
    }
    {
        vector<pair<int, int>> vy;
        for (auto& [_x, y, v] : arr) {
            if (v == 0 || v == 3) {
                vy.emplace_back(y, 0);
            } else {
                vy.emplace_back(y, 1);
            }
        }
        ans += solve(vy);
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}