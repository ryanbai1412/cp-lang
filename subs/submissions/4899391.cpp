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

template <typename T>
int first_true(int l, int r, const T& t) {
    while (l < r) {
        int mid = (l + r) / 2;
        if (t(mid)) {
            r = mid;
        } else {
            l = mid + 1;
        }
    }
    return l;
}

template <typename T>
int last_true(int l, int r, const T& t) {
    return first_true(l, r + 1,
                      [&](int x) -> bool {
                          return !t(x);
                      }) -
           1;
}

int count(const vector<array<int, 4>>& arr, int x1, int x2, int y1, int y2) {
    assert(1 <= x1 && x1 <= x2 && x2 <= 1e9 && 1 <= y1 && y1 <= y2 && y2 <= 1e9);
    int ans = 0;
    for (auto& [cx1, cx2, cy1, cy2] : arr) {
        ans += x1 <= cx1 && cx2 <= x2 && y1 <= cy1 && cy2 <= y2;
    }
    return ans;
}

vector<array<int, 4>> arr;

int raw_query(int x1, int x2, int y1, int y2) {
    cout << "? " << x1 << " " << x2 << " " << y1 << " " << y2 << endl;
#ifdef LOCAL
    return count(arr, x1, x2, y1, y2);
#else
    int ans;
    cin >> ans;
    return ans;
#endif
}

void solve() {
    int n;
    cin >> n;
#ifdef LOCAL
    arr.resize(n);
    for (auto& [x1, x2, y1, y2] : arr) {
        cin >> x1 >> x2 >> y1 >> y2;
    }
#endif
    vector<array<int, 4>> rects;
    auto query = [&](int x1, int x2, int y1, int y2) -> int {
        int ans = raw_query(x1, x2, y1, y2) - count(rects, x1, x2, y1, y2);
        assert(ans >= 0);
        return ans;
    };
    while (sz(rects) < n) {
        int x2 = first_true(1, 1e9, [&](int x2) -> bool {
            return query(1, x2, 1, 1e9);
        });
        int x1 = last_true(1, x2, [&](int x1) -> bool {
            return query(x1, x2, 1, 1e9);
        });
        int y2 = first_true(1, 1e9, [&](int y2) -> bool {
            return query(x1, x2, 1, y2);
        });
        int y1 = last_true(1, y2, [&](int y1) -> bool {
            return query(x1, x2, y1, y2);
        });
        rects.insert(rects.end(), query(x1, x2, y1, y2), {x1, x2, y1, y2});
    }
    cout << "!" << endl;
    for (auto& [x1, x2, y1, y2] : rects) {
        cout << x1 << " " << x2 << " " << y1 << " " << y2 << endl;
    }
}

int main() {
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}