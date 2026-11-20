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

struct R {
    int machine, time;
};

struct DS {
    vector<pair<int, int>> segs;

    bool isect(int l1, int r1, int l2, int r2) {
        if (l1 > l2) {
            return isect(l2, r2, l1, r1);
        }
        return r1 > l2;
    }

    int insert(int min_t, int total_t) {
        for (int i = min_t;; i++) {
            for (auto& [l, r] : segs) {
                if (isect(l, r, i, i + total_t)) {
                    goto loop;
                }
            }
            segs.emplace_back(i, i + total_t);
            return i + total_t;
        loop:;
        }
    }

    int query_end_time() const {
        int ans = 0;
        for (auto& a : segs) {
            ans = max(ans, a.second);
        }
        return ans;
    }
};

void solve() {
    int m, n;
    cin >> m >> n;
    int arr[n * m];
    for (auto& a : arr) {
        cin >> a;
        a--;
    }
    R reqs[n][m];
    for (auto& a : reqs) {
        for (auto& b : a) {
            cin >> b.machine;
            b.machine--;
        }
    }
    for (auto& a : reqs) {
        for (auto& b : a) {
            cin >> b.time;
        }
    }
    DS ds[n];
    int min_t[n] {}, req_ind[n] {};
    for (auto& a : arr) {
        auto& req = reqs[a][req_ind[a]++];
        min_t[a] = ds[req.machine].insert(min_t[a], req.time);
    }
    int ans = 0;
    for (auto& a : ds) {
        ans = max(ans, a.query_end_time());
    }
    cout << ans << endl;
}

int main() {
    cin.tie(nullptr);
    cin.exceptions(ios::failbit);
    ios_base::sync_with_stdio(false);
    solve();
}