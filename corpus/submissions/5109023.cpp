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

int Guess(int G);

int HC(int n) {
    int l = 1, r = n, last = -1;
    auto guess = [&](int x) -> int {
        last = x;
        return Guess(x);
    };
    while (l < r) {
        if (last == r + 1) {
            r = last;
        } else if (last == l - 1) {
            l = last;
        }
        int mid = (l + r) / 2, res;
        if (l == last) {
            res = guess(r);
        } else if (r == last) {
            res = -guess(l);
        } else {
            if ((r - l + 1) & 1) {
                guess(mid - 1);
            } else {
                guess(mid);
            }
            res = guess(mid + 1);
        }
        if (res < 0) {
            r = mid;
        } else if (res == 0) {
            return mid;
        } else {
            l = mid + 1;
        }
    }
    return l;
}