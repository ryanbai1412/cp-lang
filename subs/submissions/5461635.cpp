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
    int n, q;
    cin >> n >> q;
    ld arr[n][2];
    for (int i = 0; i < 2; i++) {
        for (auto& a : arr) {
            cin >> a[i];
        }
    }
    while (q--) {
        int t, l, r;
        cin >> t >> l >> r;
        l--;
        r--;
        if (t == 1) {
            ld sx = 0, sy = 0;
            for (int i = l; i <= r; i++) {
                auto& [x, y] = arr[i];
                sx += x;
                sy += y;
            }
            ld mx = sx / (r - l + 1), my = sy / (r - l + 1);
            ld num = 0, den = 0;
            for (int i = l; i <= r; i++) {
                auto& [x, y] = arr[i];
                num += (x - mx) * (y - my);
                den += (x - mx) * (x - mx);
            }
            cout << num / den << endl;
        } else if (t == 2) {
            int xadd, yadd;
            cin >> xadd >> yadd;
            for (int i = l; i <= r; i++) {
                auto& [x, y] = arr[i];
                x += xadd;
                y += yadd;
            }
        } else {
            assert(t == 3);
            int xset, yset;
            cin >> xset >> yset;
            xset++;
            yset++;
            for (int i = l; i <= r; i++) {
                auto& [x, y] = arr[i];
                x = xset + i;
                y = yset + i;
            }
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    cout << fixed << setprecision(5);
    solve();
}