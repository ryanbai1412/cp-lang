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

namespace s1 {

bool solve(vector<int> arr, const vector<pair<int, int>>& queries) {
    if (!(sz(arr) <= 50000 && sz(queries) <= 100)) {
        return false;
    }
    int n = sz(arr);
    for (auto& [ind, x] : queries) {
        sort(begin(arr), end(arr), greater<>());
        for (int i = 0; i < ind; i++) {
            arr[i] -= x;
        }
    }
    sort(begin(arr), end(arr), greater<>());
    for (int i = 0; i < n; i++) {
        cout << arr[i] << " \n"[i == n - 1];
    }
    return true;
}

}  // namespace s1

namespace s2 {

constexpr int maxn = 1005;

bool solve(vector<int> arr, const vector<pair<int, int>>& queries) {
    if (*max_element(begin(arr), end(arr)) >= maxn) {
        return false;
    }
    int n = sz(arr), cnt[maxn] {};
    for (auto& a : arr) {
        cnt[a]++;
    }
    for (auto& [ind, x] : queries) {
        int ncnt[maxn] {}, cleft = ind;
        for (int i = maxn - 1; i >= 0 && cleft; i--) {
            int csub = min(cnt[i], cleft);
            cnt[i] -= csub;
            cleft -= csub;
            ncnt[i - x] += csub;
        }
        for (int i = 0; i < maxn; i++) {
            cnt[i] += ncnt[i];
        }
    }
    for (int i = maxn - 1, j = 0; i >= 0; i--) {
        for (int k = 0; k < cnt[i]; k++, j++) {
            cout << i << " \n"[j == n - 1];
        }
    }
    return true;
}

}  // namespace s2

namespace s3 {

bool solve(const vector<int>& arr, const vector<pair<int, int>>& queries) {
    for (auto& [ind, _x] : queries) {
        if (ind != 1) {
            return false;
        }
    }
    int n = sz(arr);
    priority_queue<int> pq(begin(arr), end(arr));
    for (auto& [_ind, x] : queries) {
        int cx = pq.top();
        pq.pop();
        pq.push(cx - x);
    }
    for (int i = 0; i < n; i++) {
        cout << pq.top() << " \n"[i == n - 1];
        pq.pop();
    }
    return true;
}

}  // namespace s3

void solve() {
    int n, q;
    cin >> n >> q;
    vector<int> arr(n);
    for (auto& a : arr) {
        cin >> a;
    }
    vector<pair<int, int>> queries(q);
    for (auto& [ind, x] : queries) {
        cin >> x >> ind;
    }
    bool b = s1::solve(arr, queries) || s2::solve(arr, queries) ||
             s3::solve(arr, queries);
    assert(b);
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}