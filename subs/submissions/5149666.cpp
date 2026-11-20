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

constexpr int vals[101] = {
    0,  0,  1,  2,  2,  7,   3,  5,  4,  5,  10, 9,  9,  15, 5,   12, 14,
    17, 15, 17, 17, 23, 24,  27, 22, 34, 18, 28, 41, 29, 21, 34,  33, 41,
    26, 55, 30, 42, 35, 39,  34, 56, 65, 39, 41, 47, 45, 43, 40,  40, 29,
    44, 58, 56, 52, 57, 55,  54, 55, 48, 61, 60, 60, 58, 44, 67,  67, 64,
    64, 37, 66, 64, 74, 70,  72, 47, 66, 75, 74, 73, 90, 81, 79,  80, 78,
    81, 87, 95, 95, 83, 124, 92, 93, 86, 67, 91, 97, 88, 97, 100, 103};

void solve() {
    int n, m;
    cin >> n >> m;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }
    int cv = 0;
    for (auto& a : arr) {
        cv ^= vals[a];
    }
    for (int i = 1; i <= m; i++) {
        if (!(cv ^ vals[i])) {
            cout << i << endl;
            return;
        }
    }
    cout << -1 << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}