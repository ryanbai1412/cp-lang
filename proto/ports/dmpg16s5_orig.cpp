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
    int n, kv;
    cin >> n >> kv;
    long arr[3 * n];
    for (int i = 0; i < n; i++) {
        cin >> arr[i];
        arr[i + n] = arr[i + 2 * n] = arr[i];
    }
    n *= 3;
    long psum[n];
    partial_sum(arr, arr + n, psum);
    deque<pair<int, long>> q;
    q.emplace_back(-1, 0);
    long ans = 0;
    for (int i = 0; i < n; i++) {
        while (q.front().first < i - kv) {
            q.pop_front();
        }
        dbg(i, psum[i], q.front().first, q.front().second);
        ans = max(ans, psum[i] - q.front().second);
        while (sz(q) && q.back().second >= psum[i]) {
            q.pop_back();
        }
        q.emplace_back(i, psum[i]);
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}