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

namespace __gnu_pbds {

template <typename T>
using ost =
    tree<T, null_type, less<T>, rb_tree_tag, tree_order_statistics_node_update>;

}

using __gnu_pbds::ost;

void solve() {
    int n, kv, ql, qr;
    cin >> n >> kv >> ql >> qr;

    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }

    long psum[n + 1];
    psum[0] = 0;
    for (int i = 0; i < n; i++) {
        psum[i + 1] = psum[i] + arr[i];
    }

    auto check = [&](long x) -> long {
        long ans = 0;

        ost<pair<long, int>> ost;
        for (int i = 0; i <= n; i++) {
            if (i - ql >= 0) {
                ost.insert({psum[i - ql], i - ql});
            }
            if (i - qr - 1 >= 0) {
                ost.erase({psum[i - qr - 1], i - qr - 1});
            }

            ans += ost.order_of_key({psum[i] - x, -1});
        }

        return ans;
    };

    long l = -1e10, r = 1e10;
    while (l < r) {
        long mid = (l + r) >> 1;
        if (check(mid) < kv) {
            r = mid;
        } else {
            l = mid + 1;
        }
    }

    long x = l, ans = 0;
    int used = 0;

    set<pair<long, int>> s;
    for (int i = 0; i <= n; i++) {
        if (i - ql >= 0) {
            s.emplace(psum[i - ql], i - ql);
        }
        if (i - qr - 1 >= 0) {
            s.erase({psum[i - qr - 1], i - qr - 1});
        }

        for (auto it = s.begin(); it != s.end() && it->first < psum[i] - x;
             ++it) {
            ans += psum[i] - it->first;
            used++;
        }
    }

    ans += (kv - used) * x;
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}