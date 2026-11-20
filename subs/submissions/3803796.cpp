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
	int lc, rc, lcx, rcx;
	bool leaf;
} heap[maxn];

struct Mem {
	int hind = 1;

	int operator()() {
		auto &[lc, rc, lcx, rcx, leaf] = heap[hind];
		lc = rc = lcx = rcx = leaf = 0;
		return hind++;
	}

	void clear() {
		hind = 1;
	}
} alloc;

bool ans = true;
int arr[maxn][2];
bool leaf[maxn];

void pdfs(int i) {
	if(!i) {
		return;
	}
	leaf[i] = false;
	auto &[lc, rc] = arr[i];
	if(!lc && !rc) {
		leaf[i] = true;
	}
	pdfs(lc);
	pdfs(rc);
}

void tdfs(int i, int &o) {
	if(!i) {
		return;
	}
	if(!o) {
		o = alloc();
	}
	bool hl = !!arr[i][0];
	bool hr = !!arr[i][1];
	if(!hl && !hr) {
		heap[o].leaf = true;
	}else if(hl && hr) {
		if(leaf[arr[i][1]]) {
			tdfs(arr[i][0], heap[o].lcx);
		}
		if(leaf[arr[i][0]]) {
			tdfs(arr[i][1], heap[o].rcx);
		}
	}else if(hl) {
		tdfs(arr[i][0], heap[o].lc);
	}else {
		tdfs(arr[i][1], heap[o].rc);
	}
}

void dfs(int o) {
	if(!o) {
		ans = false;
	}
	if(!ans) {
		return;
	}
	auto &[lc, rc, lcx, rcx, leaf] = heap[o];
	if(leaf) {
		return;
	}
	dfs(lc);
	dfs(rc);
	dfs(lcx);
	dfs(rcx);
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
		pdfs(1);
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