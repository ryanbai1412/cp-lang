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

using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

namespace __gnu_pbds {

template <typename T>
using ost =
    tree<T, null_type, less<T>, rb_tree_tag, tree_order_statistics_node_update>;

}

using __gnu_pbds::ost;

struct DS {
    u64 v[4][1 << 18] {};

    void toggle(int ind) {
        ind += 10;

        v[3][ind >> 6] ^= u64(1) << (ind & 63);
        ind >>= 6;
        for (int i = 2; i >= 0; i--) {
            v[i][ind >> 6] &= ~(u64(1) << (ind & 63));
            v[i][ind >> 6] |= u64(!!v[i + 1][ind]) << (ind & 63);
            ind >>= 6;
        }
    }

    int query_gt(int ind) {
        ind += 10;

        for (int i = 3; i >= 0; i--) {
            u64 cx = v[i][ind >> 6] >> (ind & 63);
            if (cx <= 1) {
                ind >>= 6;
                continue;
            }

            int nx = (ind & 63) + 1 + __builtin_ctz(cx >> 1);
            ind = (ind & (~63)) | nx;
            for (int j = i + 1; j <= 3; j++) {
                ind = (ind << 6) | __builtin_ctz(v[j][ind]);
            }

            return ind - 10;
        }

        return -1e9;
    }
};

struct DS2 {
    set<int> s;

    void toggle(int ind) {
        auto [it, inserted] = s.insert(ind);
        if (!inserted) {
            s.erase(it);
        }
    }

    int query_gt(int ind) {
        auto it = s.upper_bound(ind);
        if (it == s.end()) {
            return -1e9;
        }
        return *it;
    }
};

void solve() {
    {
        DS ds;
        ds.toggle(2);
        ds.toggle(14);
        ds.toggle(12);
        ds.toggle(1412);

        for (int i = 0; i < 20; i++) {
            dbg(i, ds.query_gt(i));
        }
        dbg(ds.query_gt(1412));
        ds.toggle(12);
        ds.toggle(1412);
        dbg(ds.query_gt(11), ds.query_gt(12), ds.query_gt(30));
        // return;
    }
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

    vector<pair<long, int>> s_psum;
    for (int i = 0; i <= n; i++) {
        s_psum.emplace_back(psum[i], i);
    }
    sort(begin(s_psum), end(s_psum));

    int i_psum[n + 1];
    for (int i = 0; i <= n; i++) {
        i_psum[s_psum[i].second] = i;
    }

    auto check = [&](long x) -> bool {
        long ans = 0;

        DS2 ds;
        for (int i = 0; i <= n; i++) {
            if (i - ql >= 0) {
                ds.toggle(i_psum[i - ql]);
            }
            if (i - qr - 1 >= 0) {
                ds.toggle(i_psum[i - qr - 1]);
            }

            int j = -1;
            while (ans < kv) {
                j = ds.query_gt(j);
                if (j < 0) {
                    break;
                }

                if (psum[i] - s_psum[j].first > x) {
                    ans++;
                } else {
                    break;
                }
            }

            if (ans >= kv) {
                return false;
            }
        }

        return true;
    };

    long l = -1e10, r = 1e10;
    while (l < r) {
        long mid = (l + r) >> 1;
        if (check(mid)) {
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