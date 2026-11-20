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
#define dbg(...)                                              \
    cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]: "; \
    dbgh(__VA_ARGS__);
#else
#define dbg(...)
#define cerr   \
    if (false) \
    cerr
#endif

#define endl "\n"
#define long int64_t
#define sz(x) int((x).size())

extern "C" int
putaway(int n, int m, int k, int ax[], int ay[], int tx[], int ty[]) {
    int mx = *max_element(ax, ax + n), my = *max_element(ay, ay + m);
    for (int i = 0; i < k; i++) {
        if (tx[i] >= mx && ty[i] >= my) {
            return -1;
        }
    }
    vector<tuple<int, int>> evts;
    for (int i = 0; i < k; i++) {
        evts.emplace_back(tx[i], ty[i]);
    }
    for (int i = 0; i < n; i++) {
        evts.emplace_back(ax[i], 0);
    }
    sort(begin(evts), end(evts));
    sort(ay, ay + m, greater<>());
    auto check = [&](int x) -> bool {
        priority_queue<int> pq;
        for (auto& [_, ind] : evts) {
            if (ind) {
                pq.push(ind);
            } else {
                for (int i = 0; i < x && sz(pq); i++) {
                    pq.pop();
                }
            }
        }
        for (int i = 0; i < m && sz(pq); i++) {
            int cx = ay[i];
            for (int j = 0; j < x && sz(pq); j++) {
                if (pq.top() >= cx) {
                    return false;
                }
                pq.pop();
            }
        }
        return !sz(pq);
    };
    int l = 0, r = k;
    while (l < r) {
        int mid = (l + r) / 2;
        if (check(mid)) {
            r = mid;
        } else {
            l = mid + 1;
        }
    }
    return l;
}