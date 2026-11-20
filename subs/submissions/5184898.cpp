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

constexpr int sticks[10] = {6, 2, 5, 5, 4, 5, 6, 3, 7, 6};

int get_sticks(int x) {
    if (!x) {
        return sticks[0];
    }
    int ans = 0;
    while (x) {
        ans += sticks[x % 10];
        x /= 10;
    }
    return ans;
}

void solve() {
    int n;
    cin >> n;
    int ans = 0;
    for (int a = 0; a < 5000; a++) {
        for (int b = 0; b < 5000; b++) {
            if (get_sticks(a) + get_sticks(b) + get_sticks(a + b) + 4 == n) {
                ans++;
            }
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