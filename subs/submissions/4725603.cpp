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
 
const int bsize = 141;
 
int use_machine(std::vector<int> x);
 
int count_mushrooms(int n) {
    vector<int> inds[2];
    int i = 1;
    for (; i < n && max(sz(inds[0]), sz(inds[1])) < 2; i++) {
        inds[use_machine({0, i})].push_back(i);
    }
    for (; i < n && max(sz(inds[0]), sz(inds[1])) < bsize; i += 2) {
        int cind;
        if (sz(inds[0]) >= 2) {
            cind = 0;
        } else {
            cind = 1;
        }
        if (i + 2 <= n) {
            int x = use_machine({inds[cind][0], i, inds[cind][1], i + 1});
            if (x == 0) {
                inds[cind].push_back(i);
                inds[cind].push_back(i + 1);
            } else if (x == 1) {
                inds[cind].push_back(i);
                inds[cind ^ 1].push_back(i + 1);
            } else if (x == 2) {
                inds[cind ^ 1].push_back(i);
                inds[cind].push_back(i + 1);
            } else {
                inds[cind ^ 1].push_back(i);
                inds[cind ^ 1].push_back(i + 1);
            }
        } else {
            inds[use_machine({0, i})].push_back(i);
        }
    }
    int ans = sz(inds[0]);
    for (; i < n; i += bsize) {
        int cind;
        if (sz(inds[0]) >= bsize) {
            cind = 0;
        } else {
            cind = 1;
        }
        vector<int> cur;
        for (int j = i; j < n && j < i + 141; j++) {
            cur.push_back(inds[cind][j - i]);
            cur.push_back(j);
        }
        int x = (use_machine(cur) + 1) / 2;
        if (!cind) {
            ans += sz(cur) / 2 - x;
        } else {
            ans += x;
        }
    }
    return ans + 1;
}