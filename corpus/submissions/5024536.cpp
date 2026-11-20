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
    int n, m;
    cin >> n >> m;
    int arr[m][2];
    for (auto& [x, y] : arr) {
        cin >> x >> y;
        x--;
        y--;
    }
    int ans = 0, perm[n];
    iota(perm, perm + n, 0);
    do {
        for (auto& [x, y] : arr) {
            if (perm[x] > perm[y]) {
                goto loop;
            }
        }
        ans++;
    loop:;
    } while (next_permutation(perm, perm + n));
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}