#include "bits/stdc++.h"

using namespace std;

//sad; long long double doesn't exist
using ld = long double;

//imagine a language where int = long
#define long long long

//typing too hard
#define endl "\n"

#define sz(x) int(x.size())

const int maxn = 150000, bsize = 400, m = maxn / bsize + 1;

int n, l, cnt = 0;
int arr[maxn];
vector<int> buc[m];
vector<pair<int, int>> vals[m];

void maintain(int ind) {
	int n = sz(buc[ind]);
	vals[ind].resize(n);
	int j = n - 1;
	for(int i = n - 1; i >= 0; i--) {
		while(j >= 0 && buc[ind][j] > buc[ind][i] + l) {
			j--;
		}
		if(j + 1 == n) {
			vals[ind][i] = {buc[ind][i], 1};
		}else {
			vals[ind][i] = vals[ind][j + 1];
			vals[ind][i].second++;
		}
	}
}

void merge() {
	int ind = 0;
	for(auto &i: buc) {
		for(auto &j: i) {
			arr[ind++] = j;
		}
	}
}

void build() {
	for(int i = 0, ind = 0; i < n; i += bsize, ind++) {
		buc[ind].clear();
		buc[ind].insert(buc[ind].end(), arr + i, arr + min(n, i + bsize));
	}
	for(int i = 0; i < m; i++) {
		maintain(i);
	}
}

void init(int _n, int _l, int _arr[]) {
	n = _n;
	l = _l;
	for(int i = 0; i < n; i++) {
		arr[i] = _arr[i];
	}
	build();
}

void remove(int x) {
	for(int i = 0; i < m; i++) {
		if(sz(buc[i]) && buc[i].back() >= x) {
			buc[i].erase(lower_bound(begin(buc[i]), end(buc[i]), x));
			maintain(i);
			return;
		}
	}
	assert(false);
}

void insert(int x) {
	int last = -1;
	for(int i = 0; i < m; i++) {
		if(sz(buc[i])) {
			last = i;
			if(buc[i].back() >= x) {
				buc[i].insert(lower_bound(begin(buc[i]), end(buc[i]), x), x);
				maintain(i);
				return;
			}
		}
	}
	assert(last != -1);
	buc[last].push_back(x);
	maintain(last);
}

int update(int ind, int x) {
	if(++cnt == bsize) {
		merge();
		build();
	}
	remove(arr[ind]);
	insert(x);
	arr[ind] = x;
	int prev = INT_MIN;
	int ans = 0;
	for(int i = 0; i < m; i++) {
		auto it = lower_bound(begin(buc[i]), end(buc[i]), prev + l + 1);
		if(it != buc[i].end()) {
			auto cur = vals[i][it - buc[i].begin()];
			ans += cur.second;
			prev = cur.first;
		}
	}
	return ans;
}

int main() {
	cin.tie(0)->sync_with_stdio(0);
	cin.exceptions(ios::failbit);
#ifdef LOCAL
	freopen("input.txt", "r", stdin);
#endif
	int n, l;
	cin >> n >> l;
	int arr[n];
	for(int i = 0; i < n; i++) {
		cin >> arr[i];
	}
	init(n, l, arr);
	int q;
	cin >> q;
	while(q--) {
		int ind, x;
		cin >> ind >> x;
		cout << update(ind, x) << endl;
	}
}