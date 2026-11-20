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
	assert(k <= ways[tot]);
	long req = k - ways[tot - 1];
	int l = 0, r = tot;
	while (l < r) {
		int mid = (l + r) / 2;
		if (long(mid) * (tot - mid - 1) >= req) {
			r = mid;
		} else {
			l = mid + 1;
		}
	}
	r = tot - l - 1;
	if (long(l) * r >= req) {
		ans.push_back(l);
		solve(x + 1, k - long(l) * r);
	} else {
		assert(false);
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