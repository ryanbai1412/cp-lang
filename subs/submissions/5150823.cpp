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

void solve() {
    int _dim, n;
    cin >> _dim >> _dim >> n;
    vector<array<int, 5>> arr(n);
    for (int i = 0; i < n; i++) {
        auto& [x1, y1, x2, y2, qi] = arr[i];
        cin >> x1 >> y2 >> x2 >> y1;
        qi = i;
    }
    int q;
    cin >> q;
    while (q--) {
        int x, y;
        cin >> x >> y;
        for (int i = n - 1; i >= 0; i--) {
            auto& [x1, y1, x2, y2, qi] = arr[i];
            if (x1 <= x && x <= x2 && y1 <= y && y <= y2) {
                cout << qi + 1 << endl;
                arr.push_back(arr[i]);
                arr.erase(arr.begin() + i);
                goto loop;
            }
        }
        cout << 0 << endl;
    loop:;
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}