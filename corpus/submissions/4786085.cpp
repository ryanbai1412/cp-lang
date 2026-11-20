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
#define sz(x) int(size(x))

void move_inside(int i);
void move_outside(int i);
int press_button();

int min_cardinality(int n) {
    mt19937 cowng(chrono::steady_clock::now().time_since_epoch().count());
    int perm[n];
    iota(perm, perm + n, 0);
    shuffle(perm, perm + n, cowng);
    set<int> s;
    auto move_in = [&](int i) -> void {
        move_inside(perm[i]);
        assert(!s.count(i));
        s.insert(i);
    };
    auto move_out = [&](int i) -> void {
        move_outside(perm[i]);
        assert(s.count(i));
        s.erase(i);
    };
    auto press_but = [&]() -> int {
        assert(sz(s));
        return press_button();
    };
    for (int i = 0; i < n; i++) {
        move_in(i);
        if (press_but() > 1) {
            move_out(i);
        }
    }
    int distinct_cnt = sz(s);
    int l = 1, r = n / distinct_cnt;
    set<int> pool;
    for (int i = 0; i < n; i++) {
        pool.insert(i);
    }
    while (l < r) {
        assert(sz(s) == distinct_cnt * l);
        const int p = 50;
        int mid = max(l + 1, (p * l + (100 - p) * r) / 100);
        vector<int> outside, changed;
        for (auto& a : pool) {
            if (!s.count(a)) {
                outside.push_back(a);
            }
        }
        for (int i = 0; i < sz(outside) && sz(s) != distinct_cnt * mid; i++) {
            int x = outside[i];
            move_in(x);
            if (press_but() > mid) {
                move_out(x);
            } else {
                changed.push_back(x);
            }
        }
        if (sz(s) == distinct_cnt * mid) {
            l = mid;
        } else {
            r = mid - 1;
            pool = s;
            for (auto& a : changed) {
                move_out(a);
            }
        }
    }
    return l;
}