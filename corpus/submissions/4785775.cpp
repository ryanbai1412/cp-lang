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
    set<int> s;
    auto move_in = [&](int i) -> void {
        move_inside(i);
        assert(!s.count(i));
        s.insert(i);
    };
    auto move_out = [&](int i) -> void {
        move_outside(i);
        assert(s.count(i));
        s.erase(i);
    };
    for (int i = 0; i < n; i++) {
        move_in(i);
        if (press_button() > 1) {
            move_out(i);
        }
    }
    int distinct_cnt = sz(s);
    int l = 1, r = n / distinct_cnt;
    while (l < r) {
        int mid = (l + r + 1) / 2;
        auto start_snapshot = s;
        for (int i = 0; i < n; i++) {
            if (!s.count(i)) {
                move_in(i);
                if (press_button() > mid) {
                    move_out(i);
                }
            }
        }
        if (sz(s) == distinct_cnt * mid) {
            l = mid;
        } else {
            r = mid - 1;
            for (auto& a : s) {
                if (!start_snapshot.count(a)) {
                    move_out(a);
                }
            }
        }
    }
    return l;
}