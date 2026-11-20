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

int press(string p);

const string chars = "ABXY";

char solve1(string s) {
    string poss;
    if (press(s + 'A' + s + 'B') > sz(s)) {
        poss = "AB";
    } else {
        poss = "XY";
    }
    if (press(s + poss[0]) > sz(s)) {
        return poss[0];
    } else {
        return poss[1];
    }
}

string guess_sequence(int n) {
    char first = solve1("");
    string cchars;
    for (auto& a : chars) {
        if (a != first) {
            cchars += a;
        }
    }
    string s {first};
    if (n == 1) {
        return s;
    }
    for (int i = 1; i < n - 1; i++) {
        string buf = s + cchars[0];
        for (auto& a : cchars) {
            buf += s + cchars[1] + a;
        }
        dbg(buf, sz(buf), n);
        int cans = press(buf);
        if (cans == sz(s)) {
            s += cchars[2];
        } else if (cans == sz(s) + 1) {
            s += cchars[0];
        } else {
            s += cchars[1];
        }
    }
    return s + solve1(s);
}