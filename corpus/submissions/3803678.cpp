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

const int maxn = 2e6;

struct Node {
	int lc, rc;
	bool status[4];
} heap[maxn];

struct Mem {
	int hind = 1;

	int operator()() {
		auto &[lc, rc, status] = heap[hind];
		lc = rc = 0;
		memset(status, 0, sizeof(status));
		return hind++;
	}

	void clear() {
		hind = 1;
	}
} alloc;

bool ans = true;
int arr[maxn][2];

void tdfs(int i, int &o) {
	if(!i) {
		return;
	}
	if(!o) {
		o = alloc();
	}
	bool hl = !!arr[i][0];
	bool hr = !!arr[i][1];
	heap[o].status[(hl << 1) | hr] = true;
	tdfs(arr[i][0], heap[o].lc);
	tdfs(arr[i][1], heap[o].rc);
}

void dfs(int o) {
	if(!ans) {
		return;
	}
	if(heap[o].status[0]) {
		return;
	}else if(heap[o].status[1] && heap[o].status[2]) {
		dfs(heap[o].lc);
		dfs(heap[o].rc);
	}else {
		ans = false;
	}
}

void solve() {
	ans = true;
	alloc.clear();
	int root = alloc();
	int q;
	cin >> q;
	while(q--) {
		int n;
		cin >> n;
		for(int i = 1; i <= n; i++) {
			cin >> arr[i][0] >> arr[i][1];
		}
		tdfs(1, root);
	}
	dfs(root);
	if(ans) {
		cout << "Almost Complete" << endl;
	}else {
		cout << "No" << endl;
	}
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