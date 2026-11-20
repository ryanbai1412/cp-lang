#undef NDEBUG
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
int arr[maxn], order[maxn];
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
	assert(ind == n);
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
		arr[i] = order[i] = _arr[i];
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
		cnt = 0;
	}
	remove(order[ind]);
	insert(x);
	order[ind] = x;
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

struct solver {
	int n, l;
	int arr[maxn];

	solver(int n, int l, int arr[]): n(n), l(l) {
		for(int i = 0; i < n; i++) {
			this->arr[i] = arr[i];
		}
	}

	int update(int ind, int x) {
		arr[ind] = x;
		vector<int> tmp(arr, arr + n);
		sort(begin(tmp), end(tmp));
		int ans = 0, prev = INT_MIN;
		for(auto &i: tmp) {
			if(i > prev + l) {
				prev = i;
				ans++;
			}
		}
		return ans;
	}
};

void reset() {
	cnt = 0;
	memset(arr, 0, sizeof(arr));
	for(int i = 0; i < m; i++) {
		buc[i].clear();
		vals[i].clear();
	}
}

void stress() {
	mt19937 mt(chrono::steady_clock::now().time_since_epoch().count());
	auto rint = [&mt](int l, int r) {return l + mt() % (r - l + 1);};
	while(true) {
		reset();
		int n = rint(2, 5), l = rint(0, 100), q = rint(2, 5);
		int arr[n];
		for(int i = 0; i < n; i++) {
			arr[i] = rint(0, 100);
		}
		sort(arr, arr + n);
		init(n, l, arr);
		solver s(n, l, arr);
		vector<pair<int, int>> qs;
		while(q--) {
			int ind = rint(0, n - 1), x = rint(0, 100);
			qs.emplace_back(ind, x);
			int a = update(ind, x), b = s.update(ind, x);
			if(a != b) {
				cerr << n << " " << l << endl;
				for(auto &i: arr) {
					cerr << i << " ";
				}
				cerr << endl;
				for(auto &i: qs) {
					cerr << i.first << " " << i.second << endl;
				}
				cerr << a << " " << b << endl << endl;
			}
		}
	}
}

void grade() {
	reset();
	int n, l, q;
	cin >> n >> l >> q;
	int arr[n];
	for(int i = 0; i < n; i++) {
		scanf("%d", &arr[i]);
	}
	int ii[q], yy[q], sol[q];
	for(int i = 0; i < q; i++) {
		scanf("%d%d%d", &ii[i], &yy[i], &sol[i]);
	}

	init(n, l, arr);
	for(int i = 0; i < q; i++) {
		int ans = update(ii[i], yy[i]);
		if(ans != sol[i]) {
			printf("Incorrect.  In %d-th move, answered %d (%d expected).\n", i + 1, ans, sol[i]);
			return;
		}
	}
	printf("Correct.\n");
}

int main() {
//	stress();
//	freopen("elephants-test/subtask4/grader.in.1", "r", stdin);
	freopen("input.txt", "r", stdin);
	grade();
	using namespace filesystem;
	for(auto &subtask: directory_iterator("elephants-test")) {
		char last = subtask.path().string().back();
		if(isdigit(last)) {
			printf("Grading subtask %d:\n", last - '0');
			map<int, string> tests;
			for(auto &test: directory_iterator(subtask)) {
				string path = test.path().string();
				auto ind = path.find("grader.in.");
				if(ind != string::npos) {
					tests[stoi(path.substr(ind + 10))] = path;
				}
			}
			for(auto &test: tests) {
				printf("Grading case %d: ", test.first);
				freopen(test.second.c_str(), "r", stdin);
				grade();
			}
			printf("\n");
		}
	}
}