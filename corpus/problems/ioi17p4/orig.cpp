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
 
vector<int> ask(int i);
 
constexpr int maxn = 2e5 + 5;
 
struct Q {
    pair<int, int> cache[maxn];
 
    Q() {
        memset(cache, -1, sizeof(cache));
    }
 
    pair<int, int> query(int ind) {
        auto& ans = cache[ind];
        if (ans.first != -1) {
            return ans;
        }
        auto cv = ask(ind);
        assert(sz(cv) == 2);
        ans = {cv[0], cv[1]};
        return ans;
    }
} Q;
 
struct Solver {
    bool bad = false;
    int opt_ge, found_opt = -1, ans = -1;
 
    Solver(int n, int opt_ge) : opt_ge(opt_ge) {
        solve(0, n - 1, 0, 0);
    }
 
    void solve(int l, int r, int cl, int cr) {
        if (ans != -1 || bad || l > r || cl + cr == opt_ge) {
            return;
        }
        int small = 0;
        for (int mid = (l + r) / 2; mid <= r; mid++) {
            auto [ql, qr] = Q.query(mid);
            if (!ql && !qr) {
                ans = mid;
                return;
            }
            if (ql + qr < opt_ge) {
                small++;
                continue;
            }
            if (ql + qr > opt_ge) {
                bad = true;
                found_opt = max(found_opt, ql + qr);
                dbg("BAD");
                return;
            }
            dbg(mid);
            solve(mid + 1, r, ql, cr);
            solve(l, (l + r) / 2 - 1, cl, qr + small);
            return;
        }
        solve(l, (l + r) / 2 - 1, cl, cr + small);
    }
};
 
int find_best(int n) {
    int opt_ge = -1;
    while (true) {
        dbg(opt_ge);
        Solver s(n, opt_ge);
        if (s.ans != -1) {
            return s.ans;
        }
        assert(s.bad);
        opt_ge = s.found_opt;
    }
}
// The original is the function the IOI grader calls. This `main` stands in for the
// grader: n, then the hidden prize kinds (0 the diamond); ask(i) counts the boxes left
// and right of i holding a smaller kind, from each kind's sorted positions.
static vector<int> kind_of;
static vector<vector<int>> at_kind;
vector<int> ask(int i) {
	int l = 0, r = 0;
	for (int t = 0; t < kind_of[i]; t++) {
		int left = int(lower_bound(at_kind[t].begin(), at_kind[t].end(), i) - at_kind[t].begin());
		l += left;
		r += int(at_kind[t].size()) - left;
	}
	return {l, r};
}
int main() {
	int n;
	if (scanf("%d", &n) != 1) return 1;
	kind_of.resize(n);
	at_kind.assign(n, {});
	for (int i = 0; i < n; i++) {
		scanf("%d", &kind_of[i]);
		at_kind[kind_of[i]].push_back(i);
	}
	printf("%d\n", find_best(n));
}
