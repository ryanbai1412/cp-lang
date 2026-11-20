#include "bits/extc++.h"

using namespace std;

template <typename T, typename... U>
void dbgh(const T& t, const U&... u) {
    cerr << t;
    ((cerr << " | " << u), ...);
    cerr << endl;
}

#ifdef DEBUG
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__)
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int(std::size(x))

void solve() {
    int n;
    cin >> n;
    int arr[n];
    for (auto& a : arr) {
        cin >> a;
    }

    vector<int> c_inds[2];
    for (int i = 0; i < n; i++) {
        c_inds[arr[i] & 1].push_back(i);
    }

    vector<int> ans;
    if (sz(c_inds[1])) {
        ans.push_back(c_inds[1].back());
        c_inds[1].pop_back();
    }

    ans.insert(ans.end(), begin(c_inds[0]), end(c_inds[0]));
    ans.insert(ans.end(), begin(c_inds[1]), end(c_inds[1]));

    long t_ans = accumulate(arr, arr + n, long(0)) / 2;
    long psum = 0;
    for (auto& a : ans) {
        psum += arr[a];
        t_ans -= psum & 1;
    }

    cout << t_ans << endl;
    for (int i = 0; i < n; i++) {
        cout << ans[i] + 1 << " \n"[i == n - 1];
    }
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}