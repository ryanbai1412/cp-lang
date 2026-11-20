#include "bits/extc++.h"

using namespace std;

template<class T>
using mpq = priority_queue<T, vector<T>, greater<>>;

template<class T, class U = less<T>>
using rt = __gnu_pbds::tree<T, __gnu_pbds::null_type, U, __gnu_pbds::rb_tree_tag, __gnu_pbds::tree_order_statistics_node_update>;

template<class T>
void dbgh(const T &t) {
	cerr << t << endl;
}

template<class T, class ...U>
void dbgh(const T &t, const U &...u) {
	cerr << t << " | ";
	dbgh(u...);
}

#ifdef DEBUG
#define dbg(...) cerr << "L" << __LINE__ << " [" << #__VA_ARGS__ << "]" << ": "; dbgh(__VA_ARGS__)
#else
#define cerr if(false) cerr
#define dbg(...) 1412
#endif

//imagine a language where int = long
#define long int64_t

//typing too hard
#define endl "\n"

#define sz(x) static_cast<int>((x).size())

#define inline inline __attribute__((always_inline))

const int maxn = 1000, maxm = 1005;
const bool lo = false, hi = true;

int n;
vector<int> arr;
long memo[maxn][maxm][2][2];

long dp(int ind, int height, bool prev, bool cur) {
	if(!(arr[ind] <= height && height < maxm)) {
		return 4e18;
	}
	
	if(ind == n - 1) {
		if(cur == lo) {
			return arr[ind] - height;
		}
		return 4e18;
	}

	long &ans = memo[ind][height][prev][cur];
	if(ans != -1) {
		return ans;
	}
	ans = 4e18;

	// continue
	ans = min(ans, height - arr[ind + 1] + dp(ind + 1, height, cur, cur));

	// change height
	if(prev != cur) {
		int dir = cur == lo ? -1 : 1;

		ans = min(ans, dir + dp(ind, height + dir, prev, cur));
	}

	// new
	{
		int nheight;

		if(cur == lo) {
			nheight = max(height + 1, arr[ind + 1]);
		}else {
			nheight = height - 1;
		}

		ans = min(ans, nheight - arr[ind + 1] + dp(ind + 1, nheight, cur, cur ^ true));
	}

	return ans;
}

long solve(const vector<int> &_arr) {
	arr = _arr;
	n = sz(arr);
	memset(memo, -1, sizeof(memo));
	dbg(dp(0, arr[0], hi, lo));
	return dp(0, arr[0], hi, lo);
}

void solve() {
	cin >> n;
	vector<int> a(n), b(n);
	for(int i = 0; i < n; i++) {
		cin >> a[i] >> b[i];
	}
	int mx = *max_element(begin(a), end(a));
	for(auto &x: a) {
		x = mx - x;
	}
	cout << solve(a) + solve(b) << endl;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
//	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}