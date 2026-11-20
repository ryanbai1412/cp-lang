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

int last_q_floor;

bool query(int egg, int floor) {
    last_q_floor = floor;
    cout << "? " << egg << " " << floor << endl;
    bool ans;
    cin >> ans;
    return ans;
}

vector<int> iota(int n, int ind) {
    vector<int> ans(n);
    iota(begin(ans), end(ans), ind);
    return ans;
}

constexpr int maxn = 1e4 + 5;

int ans[maxn];

void solve(int fl, int fr, const vector<int>& eggs) {
    assert(fr - fl == sz(eggs));

    if (fr - fl == 1) {
        ans[eggs[0]] = fl;
        return;
    }

    int mid = (fl + fr) / 2;
    bool go_r = last_q_floor < mid - 1;
    vector<int> cc[2];

    for (auto& a : eggs) {
        cc[!query(a, mid - 1)].push_back(a);
    }

    if (go_r) {
        solve(mid, fr, cc[1]);
        solve(fl, mid, cc[0]);
    } else {
        solve(fl, mid, cc[0]);
        solve(mid, fr, cc[1]);
    }
}

void solve() {
    int n;
    cin >> n;
    solve(1, n + 1, iota(n, 1));

    cout << "! ";
    for (int i = 1; i <= n; i++) {
        cout << ans[i] << " \n"[i == n];
    }
}

int main() {
    // cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}