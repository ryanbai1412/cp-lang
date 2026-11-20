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

int query(int x, int y) {
    cout << "? " << x << " " << y << endl;
    int ans;
    cin >> ans;
    return ans;
}

void answer(int x1, int y1, int x2, int y2) {
    cout << "! " << x1 << " " << y1 << " " << x2 << " " << y2 << endl;
}

void solve() {
    int n, m;
    cin >> n >> m;
    auto ibs = [&](int x, int y) -> bool {
        return 1 <= x && x <= n && 1 <= y && y <= m;
    };
    int c1 = query(1, 1), c2 = query(n, 1), c3 = query(1, m), c4 = query(n, m);
    c1 += 2;
    c2 += -n + 1;
    c3 += -m + 1;
    c4 += -n - m;
    {
        int y1 = (c1 + c2) / 2, x1 = c1 - y1;
        if (ibs(x1, y1) && !query(x1, y1)) {
            int y2 = (c3 + c4) / -2, x2 = -c4 - y2;
            answer(x1, y1, x2, y2);
            return;
        }
    }
    int x1 = (c1 + c3) / 2, y1 = c1 - x1, x2 = (c2 + c4) / -2, y2 = -c4 - x2;
    answer(x1, y1, x2, y2);
}

int main() {
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}