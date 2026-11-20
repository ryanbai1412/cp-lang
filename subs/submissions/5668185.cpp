#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

void solve() {
    int n, q;
    cin >> n >> q;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }

    while (q--) {
        int x1, x2, y1, y2;
        cin >> x1 >> x2 >> y1 >> y2;
        x1--;
        x2--;
        y1--;
        y2--;
        int ans = 0;
        for (int i = x1; i <= x2; i++) {
            for (int j = i + 1; j <= x2; j++) {
                ans += y1 <= arr[i] && arr[i] <= y2 && y1 <= arr[j] &&
                       arr[j] <= y2 && arr[i] <= arr[j];
            }
        }
        cout << ans << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}