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

map<pair<int, int>, A> memo;

A dp(int n, int m) {
    if (n % 2 || n / 2 > m) {
        return A::c_def();
    } else if (!n || !m) {
        return A::c_zero();
    }
    auto [it, inserted] = memo.insert({{n, m}, A::c_def()});
    A& ans = it->second;
    if (!inserted) {
        return ans;
    }

    int k_base = max(0, (m - 2 * n) / (n / 2) - 1);
    dbg(n, m, k_base);
    for (int k = k_base; ; k++) {
        int nx = m - k * (n / 2);
        if (nx < 0) {
            break;
        }
        ans += dp(int(nx), n) + k;
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