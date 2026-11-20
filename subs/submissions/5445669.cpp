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
    int kv, n;
    cin >> kv >> n;
    int arr[n][3];
    for (auto& [x, y, w] : arr) {
        cin >> x >> y >> w;
    }
    int ans_mx = 0, ans_ways = 0;
    for (int i = 0; i <= 128; i++) {
        for (int j = 0; j <= 128; j++) {
            int cans = 0;
            for (auto& [x, y, w] : arr) {
                if (abs(x - i) <= kv && abs(y - j) <= kv) {
                    cans += w;
                }
            }
            if (cans > ans_mx) {
                ans_mx = cans;
                ans_ways = 1;
            } else if (cans == ans_mx) {
                ans_ways++;
            }
        }
    }
    cout << ans_ways << " " << ans_mx << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}