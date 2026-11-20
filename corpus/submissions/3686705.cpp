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

const int maxn = 5e5;

int n, color[maxn], arr[maxn][2];
long x;

long sq(long x) {
	return x * x;
}

long dist(int u, int v) {
	return sq(arr[u][0] - arr[v][0]) + sq(arr[u][1] - arr[v][1]);
}

bool dfs(int u, int c = 0) {
	if(color[u] != -1) {
		return color[u] == c;
	}
	color[u] = c;
	bool ans = true;
	for(int v = 0; v < n; v++) {
		if(dist(u, v) < x && u != v) {
			ans &= dfs(v, c ^ 1);
		}
	}
	return ans;
}

bool valid(long x) {
	::x = x;
	memset(color, -1, sizeof(color));
	for(int i = 0; i < n; i++) {
		if(color[i] == -1 && !dfs(i)) {
			return false;
		}
	}
	return true;
}

void solve() {
	cin >> n;
	for(int i = 0; i < n; i++) {
		cin >> arr[i][0] >> arr[i][1];
	}
	long l = 0, r = 4e18;
	dbg(valid(20));
	while(l < r) {
		int mid = (l + r + 1) / 2;
		if(valid(mid)) {
			l = mid;
		}else {
			r = mid - 1;
		}
	}
	cout << l << endl;
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