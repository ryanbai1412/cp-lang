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

long divf(long a, long b) {
    assert(b > 0);
    if (a >= 0) {
        return a / b;
    } else {
        a = -a;
        return -((a + b - 1) / b);
    }
}

void solve() {
    int n, q;
    cin >> n >> q;
    long arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    while (q--) {
        int ty, l, r;
        cin >> ty >> l >> r;
        l--;
        r--;
        if (ty == 1) {
            int x;
            cin >> x;
            for (int i = l; i <= r; i++) {
                arr[i] += x;
            }
        } else if (ty == 2) {
            int x;
            cin >> x;
            for (int i = l; i <= r; i++) {
                arr[i] = divf(arr[i], x);
            }
        } else if (ty == 3) {
            cout << *min_element(arr + l, arr + r + 1) << endl;
        } else {
            cout << accumulate(arr + l, arr + r + 1, long(0)) << endl;
        }
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}