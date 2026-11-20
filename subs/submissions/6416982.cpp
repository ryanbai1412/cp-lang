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

using u64 = uint64_t;

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

struct A {
    long mn, mx;

    A& operator+=(const A& o) {
        mn = min(mn, o.mn);
        mx = max(mx, o.mx);
        return *this;
    }

    A& operator+=(long x) {
        mn += x;
        mx += x;
        return *this;
    }

    friend A operator+(A a, const A& b) {
        return a += b;
    }
    friend A operator+(A a, long b) {
        return a += b;
    }
    friend A operator+(long b, A a) {
        return a + b;
    }

    static A c_def() {
        return {long(1e18), long(-1e18)};
    }

    static A c_zero() {
        return {};
    }
};

namespace memo {

constexpr int HSIZE = 1 << 23;

pair<u64, A> arr[HSIZE + (1 << 20)];

u64 to_key(u64 x) {
    x += 0x895798156719086;
    x = (x ^ (x >> 30)) * 0x8915709156124796;
    // x = (x ^ (x >> 27)) * 0x1895019568176510;
    return x ^ (x >> 31);
}

pair<A*, bool> find(u64 k) {
    u64 ck = to_key(k);

    ck &= (HSIZE - 1);
    for (; arr[ck].first && arr[ck].first != k; ck++)
        ;
    if (!arr[ck].first) {
        arr[ck].first = k;
        return {&arr[ck].second, true};
    }
    return {&arr[ck].second, false};
}

}  // namespace memo

constexpr int MAXC = 1005;

auto pcomp = []() {
    constexpr int n = MAXC, m = MAXC;

    vector dp(n + 1, vector(m + 1, A::c_def()));
    for (int i = 0; i <= n; i++) {
        dp[i][0] = A::c_zero();
    }
    for (int j = 0; j <= m; j++) {
        dp[0][j] = A::c_zero();
    }
    for (int i = 1; i <= n; i++) {
        for (int j = 1; j <= m; j++) {
            auto go = [&](int ni, int nj) -> void {
                if (ni < 0 || nj < 0) {
                    return;
                }
                dp[i][j] += dp[ni][nj] + 1;
            };
            go(i, j - 2 * i);
            go(i - 2 * j, j);
            if (i % 2 == 0) {
                go(i, j - i / 2);
            }
            if (j % 2 == 0) {
                go(i - j / 2, j);
            }
        }
    }

    return dp;
}();

A dp(int n, int m) {
    int jump = n % 2 == 0 ? n / 2 : 2 * n;
    if (jump > m) {
        return A::c_def();
    } else if (!n || !m) {
        return A::c_zero();
    } else if (n <= MAXC && m <= MAXC) {
        return pcomp[n][m];
    }
    auto [it, inserted] = memo::find((u64(n) << 30) | m);
    A& ans = *it;
    if (!inserted) {
        return ans;
    }
    ans = A::c_def();

    int k_base = max(0, (m - 2 * n) / jump - 1);
    dbg(n, m, k_base);
    for (int k = k_base;; k++) {
        int nx = m - k * jump;
        if (nx < 0) {
            break;
        }
        auto ndp = dp(int(nx), n);
        ans += ndp + k;
        if (n % 2 == 0) {
            int cntk = (k / 4) + (k % 4);
            ans += ndp + cntk;
        }
    }

    return ans;
}

void solve() {
    int n, m;
    cin >> n >> m;
    auto ans = dp(n, m) + dp(m, n);
    cout << ans.mn << " " << ans.mx << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}