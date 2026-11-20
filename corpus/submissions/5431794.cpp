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

struct H {
    uint64_t s = 0x2084567968987;

    H& operator()(uint64_t x) {
        s = s * 0x1359879857158791 + 0x0984507259687;
        s += x * 0x289527895698;
        return *this;
    }
};

constexpr long mod = 1e8 + 7;

long solve1(int a, int b, int c) {
    auto psum = [&](long x) -> long { return x * (x + 1) / 2; };
    b--;
    c--;
    a = min(a, c);
    b = min(b, c);
    long ans = long(min(a + 1, c - b)) * (b + 1);
    if (c - a + 1 <= b + 1) {
        ans += psum(b + 1) - psum(c - a);
    }
    return ans;
}

void solve() {
    int n, m, kv;
    cin >> n >> m >> kv;
    auto ibs = [&](int x, int y) -> bool {
        return 0 <= x && x <= n && 0 <= y && y <= m;
    };
    pair<int, int> arr[kv];
    for (auto& [x, y] : arr) {
        cin >> x >> y;
    }
    long ans = 0;
    for (int i = 1; i <= min(n, m); i++) {
        ans = (ans + long(n - i + 1) * (m - i + 1) % mod * i) % mod;
    }
    dbg(ans);
    for (auto& [x, y] : arr) {
        int xl = x, xr = n - x, yl = y, yr = m - y;
        long cans = solve1(xl, xr, yl) + solve1(yr, yl, xl) +
                    solve1(xr, xl, yr) + solve1(yl, yr, xr);
        dbg(cans);
        ans -= cans;
    }
    __gnu_pbds::gp_hash_table<uint64_t, int> mp;
    auto upd = [&](int x, int y, int dx, int dy) -> void {
        array<pair<int, int>, 4> sq {{{x, y},
                                      {x + dx, y + dy},
                                      {x + dx - dy, y + dx + dy},
                                      {x - dy, y + dx}}};
        for (auto& [x, y] : sq) {
            if (!ibs(x, y)) {
                return;
            }
        }
        sort(begin(sq), end(sq));
        H h;
        for (auto& [x, y] : sq) {
            h(x)(y);
        }
        mp[h.s]++;
    };
    for (int i = 0; i < kv; i++) {
        for (int j = 0; j < kv; j++) {
            if (i == j) {
                continue;
            }
            auto& [x1, y1] = arr[i];
            auto& [x2, y2] = arr[j];
            // diagonal
            if (i < j) {
                int dx2 = x2 + y2 - x1 - y1;
                int dy2 = y2 - x2 + x1 - y1;
                if (dx2 % 2 == 0) {
                    upd(x1, y1, dx2 / 2, dy2 / 2);
                }
            }
            // side
            upd(x1, y1, x2 - x1, y2 - y1);
        }
    }
    for (auto& [_k, v] : mp) {
        dbg(v);
        if (v == 1) {
            ans += 1;
        } else if (v == 3) {
            ans += 2;
        } else if (v == 6) {
            ans += 3;
        } else {
            dbg(v);
            assert(false);
        }
    }
    ans %= mod;
    ans = (ans + mod) % mod;
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}