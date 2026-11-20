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

template<typename T, typename Operation>
struct SparseTable {
	vector<vector<T>> v;
	const Operation operation;

	SparseTable(const vector<T> &arr, const Operation &operation = Operation()): operation(operation) {
		int n = sz(arr), m = __lg(n) + 1;
		v.resize(m, vector<T>(n));
		v[0] = arr;
		for(int i = 1; i < m; i++) {
			for(int j = 0; j + (1 << i) <= n; j++) {
				v[i][j] = operation(v[i - 1][j], v[i - 1][j + (1 << (i - 1))]);
			}
		}
	}

	T query(int l, int r) const {
		int x = __lg(r - l + 1);
		return operation(v[x][l], v[x][r - (1 << x) + 1]);
	}
};

template<typename T, bool cmpMin>
struct MinMaxOperator {
	T operator()(const T &a, const T &b) const {
		if(cmpMin) {
			return min(a, b);
		}
		return max(a, b);
	}
};

template<typename T>
using RangeMin = SparseTable<T, MinMaxOperator<T, true>>;

template<typename T>
using RangeMax = SparseTable<T, MinMaxOperator<T, false>>;

const int maxn = 1e6;

int cl, cr, n, arr[maxn];
RangeMin<pair<int, int>> *rmq;

int solve(int l, int r, int sub) {
	if(l > r) {
		return 0;
	}
	dbg(l, r, sub);
	auto[x, ind] = rmq->query(l, min(r, l + cr - 1));
	int change = x - sub;
	if(change && r - l + 1 < cl) {
		return -1;
	}
	int gl = solve(l, ind - 1, x), gr = solve(ind + 1, r, x);
	if(gl == -1 || gr == -1) {
		return -1;
	}
	return gl + gr + change;
}

void solve() {
	cin >> n >> cl >> cr;
	vector<pair<int, int>> cur;
	for(int i = 0; i < n; i++) {
		cin >> arr[i]; 
		cur.emplace_back(arr[i], i);
	}
	rmq = new RangeMin<pair<int, int>>(cur);
	cout << solve(0, n - 1, 0) << endl;
	delete rmq;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int t = 1;
	cin >> t;
	for(int _ = 1; _ <= t; _++) {
		dbg(_);
//		cout << "Case #" << _ << ": ";
		solve();
	}
}