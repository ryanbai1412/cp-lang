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

mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());

int query(int cap, int bottle) {
    cout << 0 << " " << cap << " " << bottle << endl;
    int ans;
    cin >> ans;
    return ans;
}

void answer(int cap, int bottle) {
    cout << 1 << " " << cap << " " << bottle << endl;
}

void solve(const vector<int>& cap, const vector<int>& bottle) {
    assert(sz(cap) == sz(bottle));
    if (!sz(cap)) {
        return;
    } else if (sz(cap) == 1) {
        answer(cap[0], bottle[0]);
        return;
    }
    int cb = bottle[cowng() % sz(bottle)], mcap = -1;
    vector<int> lcap, rcap, lbottle, rbottle;
    for (auto& a : cap) {
        int x = query(a, cb);
        if (x < 0) {
            lcap.push_back(x);
        } else if (x > 0) {
            rcap.push_back(x);
        } else {
            mcap = x;
        }
    }
    assert(mcap != -1);
    answer(mcap, cb);
    for (auto& a : bottle) {
        if (a == cb) {
            continue;
        }
        if (query(mcap, a) < 0) {
            rcap.push_back(a);
        } else {
            lcap.push_back(a);
        }
    }
    solve(lcap, lbottle);
    solve(rcap, rbottle);
}

void solve() {
    int n;
    cin >> n;
    vector<int> arr(n);
    iota(begin(arr), end(arr), 1);
    solve(arr, arr);
}

int main() {
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}