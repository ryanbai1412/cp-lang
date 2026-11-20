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

long sq(long x) {
    return x * x;
}

void solve() {
    int n;
    cin >> n;
    array<int, 3> arr[n];
    for (auto& [x, y, r] : arr) {
        cin >> x >> y >> r;
    }
    sort(arr, arr + n, [&](const auto& a, const auto& b) -> bool {
        return a[2] > b[2];
    });
    int cnt[n] {}, cand[n], *cptr = cand, par[n];
    memset(par, -1, sizeof(par));
    for (int i = n - 1; i >= 0; i--) {
        cptr = remove_if(cand, cptr, [&](int j) -> bool {
            auto& [xi, yi, ri] = arr[i];
            auto& [xj, yj, _rj] = arr[j];
            if (sq(xi - xj) + sq(yi - yj) <= sq(ri)) {
                par[j] = i;
                return true;
            }
            return false;
        });
        *(cptr++) = i;
    }
    for (int i = 0; i < n; i++) {
        if (par[i] != -1) {
            cnt[i] = cnt[par[i]] + 1;
        }
    }
    long ans = 0;
    for (int i = 0; i < n; i++) {
        long cx = sq(arr[i][2]);
        if (cnt[i] % 2 == 0) {
            ans += cx;
        } else {
            ans -= cx;
        }
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}