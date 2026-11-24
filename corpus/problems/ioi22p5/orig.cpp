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
    while (l < r) {
        int mid = max(l + 1, (4 * l + 1 * r) / 5);
        vector<int> outside, changed;
        for (int i = 0; i < n; i++) {
            if (!s.count(i)) {
                outside.push_back(i);
            }
        }
        for (int i = 0; i < sz(outside); i++) {
            if (sz(s) + sz(outside) - i < distinct_cnt * mid) {
                break;
            }
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
            for (auto& a : changed) {
                move_out(a);
            }
        }
    }
    return l;
}

// The sample grader, reading the input the port reads (corpus/submissions/4785860.cpp is the grader function).
static vector<int> grader_kind;
static vector<char> grader_inside;
static map<int, int> grader_count;
static multiset<int> grader_sizes;
static void grader_set(int t, int d) {
    if (grader_count[t]) grader_sizes.erase(grader_sizes.find(grader_count[t]));
    if (grader_count[t] += d) grader_sizes.insert(grader_count[t]);
}
void move_inside(int i) { if (!grader_inside[i]) grader_inside[i] = 1, grader_set(grader_kind[i], 1); }
void move_outside(int i) { if (grader_inside[i]) grader_inside[i] = 0, grader_set(grader_kind[i], -1); }
int press_button() { return grader_sizes.empty() ? 0 : *grader_sizes.rbegin(); }
int main() {
    int n; scanf("%d", &n);
    grader_kind.resize(n); grader_inside.assign(n, 0);
    for (auto& x : grader_kind) scanf("%d", &x);
    printf("%d\n", min_cardinality(n));
}
