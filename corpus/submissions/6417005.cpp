#pragma GCC optimize("Ofast")

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

constexpr int MAXC = 4096, HSIZE = 1 << 23;

A cache[MAXC][MAXC];
pair<u64, A> arr[HSIZE + (1 << 20)];

u64 to_key(u64 x) {
    int count = int(x >> 61);

    x *= 0x1489619615687165871;
    x ^= x >> 22;
    return x >> (22 + count);
}

pair<A*, bool> find(int n, int m) {
    if (n < MAXC && m < MAXC) {
        return {&cache[n][m], !cache[n][m].mn};
    }
    u64 k = (u64(n) << 30) | m;
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

A dp(int n, int m) {
    int jump = n % 2 == 0 ? n / 2 : 2 * n;
    if (jump > m) {
        return A::c_def();
    } else if (!n || !m) {
        return A::c_zero();
    }
    auto [it, inserted] = memo::find(n, m);
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