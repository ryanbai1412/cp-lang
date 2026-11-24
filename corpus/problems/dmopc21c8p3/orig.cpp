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
#define sz(x) int((x).size())

namespace __gnu_pbds {
template <typename T, typename U = less<T>>
using ost =
	tree<T, null_type, U, rb_tree_tag, tree_order_statistics_node_update>;
}

using __gnu_pbds::ost;

const int maxn = 1e6 + 5;
const long inf = 1e18 + 5;

long ways[maxn];

void pcomp() {
	for (int i = 1; i < maxn; i++) {
		int l = i / 2, r = i - l - 1;
		ways[i] = min(inf, ways[i - 1] + long(l) * r);
	}
}

int n;
vector<int> ans;

void solve(int x, long k) {
	if (x == n + 1) {
		return;
	}
	int tot = n - x + 1;
	if (k <= ways[tot - 1]) {
		ans.push_back(0);
		solve(x + 1, k);
	} else {
		if (tot == 5 && k == 4) {
			ans.push_back(1);
			solve(x + 1, k - 3);
		} else {
			int l = (tot - 1) / 2;
			ans.push_back(int(l));
			solve(x + 1, k - long(l) * (tot - l - 1));
		}
	}
}

void solve() {
	long k;
	cin >> n >> k;
	if (k > ways[n]) {
		cout << -1 << endl;
		return;
	}
	solve(1, k);
	int xans[n];
	ost<int> cur;
	for (int i = 0; i < n; i++) {
		cur.insert(i);
	}
	for (int i = 0; i < n; i++) {
		dbg(i, ans[i]);
		auto it = cur.find_by_order(ans[i]);
		xans[*it] = i;
		cur.erase(it);
	}
	for (int i = 0; i < n; i++) {
		cout << xans[i] + 1 << " \n"[i == n - 1];
	}
}

int main() {
	cin.tie(nullptr);
	ios_base::sync_with_stdio(false);
	cin.exceptions(ios::failbit);
	pcomp();
	solve();
}