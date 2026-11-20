#pragma GCC optimize("Ofast")
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

const int maxn = 300, maxm = 1e4;
const long mod = 1e9 + 7;

int n, arr[maxn][2];
vector<long> memo[maxn][maxn];

long dp(int l, int r, int m);

void pdp(int l, int r) {
	vector<long> cur(maxm + 1);
	auto process = [&](int i) -> void {
		for(int j = arr[i][0]; j <= arr[i][1]; j++) {
			cur[j] = cur[j] + dp(l, i - 1, j) * dp(i + 1, r, j - 1) % mod;
		}
	};
	int mid = (l + r) / 2;
	process(mid);
	process(mid + 1);
	if((r - l + 1) & 1) {
		process(mid - 1);
	}
	memo[l][r].resize(sz(cur));
	partial_sum(begin(cur), end(cur), memo[l][r].begin());
}

long dp(int l, int r, int m) {
	if(l > r) {
		return 1;
	}else if(l == r) {
		return max(0, min(arr[l][1], m) - arr[l][0] + 1);
	}
	if(!sz(memo[l][r])) {
		pdp(l, r);
	}
	return memo[l][r][m];
}

void solve() {
	cin >> n;
	for(int i = 0; i < n; i++) {
		cin >> arr[i][0] >> arr[i][1];
	}
	cout << dp(0, n - 1, maxm) << endl;
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