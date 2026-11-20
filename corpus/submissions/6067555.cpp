#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
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
    int q, m;
    cin >> q >> m;

    vector<array<int, 3>> arr;

    while (q--) {
        int ty;
        cin >> ty;

        if (ty == 1) {
            int t, c, l, r, d;
            cin >> t >> c >> l >> r >> d;

            int len = r - l;
            if (d == -1) {
                l = 2 * m - l;
            }
            l -= t;
            l %= 2 * m;
            if (l < 0) {
                l += 2 * m;
            }

            arr.push_back({c, l, len});
        } else if (ty == 2) {
            auto inter = [&](int l1, int r1, int l2, int r2) -> bool {
                if (l1 > l2) {
                    swap(l1, l2);
                    swap(r1, r2);
                }
                return r1 >= l2;
            };

            int t, ql, qr;
            cin >> t >> ql >> qr;

            int ans = 0;

            for (auto [_c, l, len] : arr) {
                l = (l + t) % (2 * m);
                if (l >= m) {
                    l = 2 * m - l;
                }
                int r = l + len;

                ans += inter(ql, qr, l, r);
            }

            cout << ans << endl;
        } else {
            int _t, c;
            cin >> _t >> c;

            arr.erase(
                remove_if(begin(arr), end(arr),
                          [&](const auto& a) -> bool { return a[0] == c; }),
                arr.end());
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}