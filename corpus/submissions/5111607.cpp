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

int n;

bool ibs(int x, int y) {
    return 0 <= x && x < n && 0 <= y && y < n;
}

bool query(int x, int y) {
    if (!ibs(x, y)) {
        return false;
    }
    cout << "examine " << x + 1 << " " << y + 1 << endl;
    string ans;
    cin >> ans;
    return ans == "true";
}

void answer(int x, int y) {
    cout << "solution " << x + 1 << " " << y + 1 << endl;
    exit(0);
}

void solve() {
    int x0, y0;
    cin >> n >> x0 >> y0;
    x0--;
    y0--;
    auto bsearch = [&](auto cb) -> int {
        int l = 0, r = 1;
        while (cb(r)) {
            r <<= 1;
        }
        while (l < r) {
            int mid = (l + r + 1) / 2;
            if (cb(mid)) {
                l = mid;
            } else {
                r = mid - 1;
            }
        }
        return l;
    };
    int xl = bsearch([&](int x) -> bool {
        return query(x0 - x, y0);
    });
    int xr = bsearch([&](int x) -> bool {
        return query(x0 + x, y0);
    });
    int m = xr - xl + 1;
    int yl = bsearch([&](int x) -> bool {
        return query(x0, y0 - x);
    });
    while (query(xl, yl)) {
        xl -= m;
        yl -= m;
    }
    answer(xl + m * 3 + m / 2, yl + m * 3 + m / 2);
}

int main() {
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}