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

struct Grader {
    int xl, yl, m;

    void init() {
        cin >> xl >> yl >> m;
    }

    bool query(int x, int y) const {
        assert(1 <= x && x <= n && 1 <= y && y <= n);
        if (xl <= x && x < xl + 5 * m && yl <= y && y < yl + 5 * m) {
            x -= xl;
            y -= yl;
            if ((x / m) % 2 == 0 && (y / m) % 2 == 0) {
                return true;
            }
        }
        return false;
    }

    void answer(int x, int y) const {
        dbg(x, y);
        assert(x == xl + m * 2 + m / 2);
        assert(y == yl + m * 2 + m / 2);
    }
} grader;

bool ibs(int x, int y) {
    return 1 <= x && x <= n && 1 <= y && y <= n;
}

bool query(int x, int y) {
    if (!ibs(x, y)) {
        return false;
    }
#ifdef LOCAL
    return grader.query(x, y);
#endif
    cout << "examine " << x << " " << y << endl;
    string ans;
    cin >> ans;
    assert(ans != "-1");
    return ans == "true";
}

void answer(int x, int y) {
#ifdef LOCAL
    grader.answer(x, y);
#endif
    cout << "solution " << x << " " << y << endl;
    exit(0);
}

void solve() {
    int x0, y0;
    cin >> n >> x0 >> y0;
#ifdef LOCAL
    grader.init();
#endif
    auto bsearch = [&](auto cb) -> int {
        int l = 0, r = 1;
        while (cb(r)) {
            r <<= 1;
        }
        dbg(cb(0), cb(1), cb(2));
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
    int xl = x0 - bsearch([&](int x) -> bool {
        return query(x0 - x, y0);
    });
    int xr = x0 + bsearch([&](int x) -> bool {
        return query(x0 + x, y0);
    });
    int m = xr - xl + 1;
    dbg(xl, xr, m);
    int yl = y0 - bsearch([&](int x) -> bool {
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