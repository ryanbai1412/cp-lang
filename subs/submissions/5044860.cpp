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

struct Range {
    int l, r;
    
    Range(): l(-1e9), r(1e9) {}
    
    void update(int xl, int xr) {
        l = max(l, xl);
        r = min(r, xr);
    }
    
    bool valid() const {
        return l <= r;
    }
};

void solve() {
    int n;
    cin >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    auto check = [&](int x) -> bool {
        Range r1, r2;
        for (int i = 0; i < n; i++) {
            int xl = i - arr[i] - x, xr = i - arr[i] + x;
            if (xl < 0) {
                xl += n;
                xr += n;
            }
            dbg(xl, xr);
            r1.update(xl, xr);
            r2.update(xl - n, xr - n);
        }
        return r1.valid() || r2.valid();
    };
    int l = 0, r = n;
    while (l < r) {
        int mid = (l + r) / 2;
        if (check(mid)) {
            r = mid;
        } else {
            l = mid + 1;
        }
    }
    cout << l << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}