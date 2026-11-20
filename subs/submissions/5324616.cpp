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

using i128 = __int128_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

constexpr int L = 1e9 + 1;

void solve() {
    int n, kl, kr;
    cin >> n >> kl >> kr;
    pair<int, bool> arr[n];
    for (auto& a : arr) {
        cin >> a.first;
    }
    for (auto& a : arr) {
        cin >> a.second;
    }
    auto rev = [&](pair<int, bool> px) -> pair<int, bool> {
        auto [p, d] = px;
        return {L - p, !d};
    };
    auto simu = [&](long x) -> tuple<int, int, vector<pair<int, bool>>> {
        long h[2], m = x % (2 * L);
        if (i128(n) * (x / (2 * L)) > i128(1e9)) {
            return {INT_MAX, INT_MAX, {}};
        }
        h[0] = h[1] = n * (x / (2 * L));
        vector<pair<int, bool>> ans;
        auto proc = [&](pair<int, bool> px, int hind) -> pair<int, bool> {
            auto [p, d] = px;
            assert(!d);
            if (m < p) {
                return {p - m, false};
            } else if (m < p + L) {
                h[hind]++;
                return {m - p, true};
            } else {
                h[0]++;
                h[1]++;
                return {p + 2 * L - m, false};
            }
        };
        for (auto [p, d] : arr) {
            if (!d) {
                ans.push_back(proc({p, d}, 0));
            } else {
                ans.push_back(rev(proc(rev({p, d}), 1)));
            }
        }
        return {h[0], h[1], ans};
    };
    simu(2000000003);
    //    return;
    long t1 = ([&]() -> long {
        long l = 0, r = 2e18;
        while (l < r) {
            long mid = (l + r) / 2;
            auto [b1, b2, _] = simu(mid);
            if (b1 >= kl || b2 >= kr) {
                r = mid;
            } else {
                l = mid + 1;
            }
        }
        return l;
    })();
    dbg(t1);
    auto [bl, br, rarr] = simu(t1);
    if (bl >= kl && br >= kr) {
        cout << t1 + L << endl;
        return;
    }
    if (br >= kr) {
        swap(bl, br);
        swap(kl, kr);
        for (auto& a : rarr) {
            a = rev(a);
        }
    }
    assert(bl);
    vector<int> mr;
    for (auto& [p, d] : rarr) {
        dbg(p, d);
        if (d) {
            mr.push_back(d);
        }
    }
    assert(sz(mr));
    sort(begin(mr), end(mr), greater<>());
    if (kr - br < sz(mr)) {
        cout << t1 + 2 * L - mr[kr - br] << endl;
    } else {
        cout << t1 + 2 * L << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}