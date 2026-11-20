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

using C = complex<double>;

constexpr int maxn = 10;

int n;
C arr[maxn][2];

int solve(int ind, C p) {
    if (ind == -1) {
        return 0 <= real(p) && real(p) <= 100 && 0 <= imag(p) && imag(p) <= 100;
    }
    auto& [c1, c2] = arr[ind];
    C v1 = c2 - c1, v2 = p - c1;
    C v3 = v1 * conj(v2);
    if (imag(v3) > 0) {
        return 0;
    }
    int ans = solve(ind - 1, p);
    if (ind == 1) {
        dbg(c1 + v1 * v3 / norm(v1));
    }
    ans += solve(ind - 1, c1 + v1 * v3 / norm(v1));
    return ans;
}

void solve() {
    cin >> n;
    for (int i = 0; i < n; i++) {
        for (auto& a : arr[i]) {
            double x, y;
            cin >> x >> y;
            a = {x, y};
        }
    }
    int q;
    cin >> q;
    while (q--) {
        double x, y;
        cin >> x >> y;
        cout << solve(n - 1, C(x, y)) << endl;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}