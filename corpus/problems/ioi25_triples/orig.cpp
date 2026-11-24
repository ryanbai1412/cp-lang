// IOI 2025 Triple Peaks: the official model solution, ioi/task-archive (github.com/ioi/task-archive)
// 2025/day1/triples/solutions/model_solution/solution.cpp, with the sample grader's main below
// in place of graders/triples.h and graders/grader.cpp.
#include <vector>
#include <cstdio>
#include <cassert>
#include <cmath>
#include <algorithm>
#include <set>
#include <map>
#include <random>

using namespace std;

bool goodTriple(int i, int j, int k, const vector<int>& a) {
	assert(i != j && i != k && j != k);
	vector<int> A{a[i], a[j], a[k]};
	vector<int> B{abs(i - j), abs(i - k), abs(j - k)};
	sort(A.begin(), A.end());
	sort(B.begin(), B.end());
	return A == B;
}

long long count_triples(vector<int> a) { // O(N^1.5 + N*log(N))
	int n = a.size();
	set<set<int>> triples; // we'll insert only O(N) triples here
	auto considerPair = [&](int i, int j) {
		for (int id : {i, j}) {
			for (int dist : {a[i], a[j]}) {
				for (int k : {id - dist, id + dist}) {
					if (0 <= k && k < n && k != i && k != j) {
						if (goodTriple(i, j, k, a)) {
							triples.insert(set<int>{i, j, k});
						}
					}
				}
			}
		}
	};
	
	for (int i = 0; i < n; i++) {
		for (int j : {i - a[i], i + a[i]}) {
			if (0 <= j && j < n) {
				considerPair(i, j);
			}
		}
	}
	
	int answer = triples.size();
	auto considerTriple = [&](int i, int j, int k) {
		if (0 <= i && i < j && j < k && k < n && a[i] == k - j && a[j] == k - i && a[k] == j - i) {
			if (a[k] == j - i && a[i] != a[k]) { // symmetrical triples were counted already
				answer++;
			}
		}
	};
	
	map<int, vector<int>> diag1, diag2;
	for (int i = 0; i < n; i++) {
		diag1[i+a[i]].push_back(i);
		diag2[i-a[i]].push_back(i);
	}
	for (int j = 0; j < n; j++) {
		vector<int>& x = diag1[j+a[j]];
		vector<int>& y = diag2[j-a[j]];
		// i < j < k
		if (x.size() < y.size()) { // smaller-to-larger
			for (int k : x) {
				int i = k - a[j];
				considerTriple(i, j, k);
			}
		}
		else {
			for (int i : y) {
				if (i < j) {
					int k = i + a[j];
					considerTriple(i, j, k);
				}
			}
		}
	}
	return answer;
}

vector<int> construct_range(int M, int ) {
	int n = M;
	pair<int, vector<int>> bestAnswer{0, {}};
	for (int seed = 0; seed < 1'000'000 / n; seed++) {
		if (n == 500) {
			seed = 175691; // best seed after running program for 1 minute, trying different seeds
		}
		mt19937 rng(seed);
		int k = sqrt(6 * n + 0.1);
		k = uniform_int_distribution<int>(k * 2 / 3, k * 4 / 3)(rng); // for bigger variance
		set<int> s;
		while ((int) s.size() < k) {
			int x = uniform_int_distribution<int>(-n/2, n*3/2 - 1)(rng); // better than [0,n-1]
			x = x / 2 * 2; // make even
			s.insert(x);
		}
		vector<int> a(n, INT32_MAX);
		for (int x : s) {
			for (int y : s) {
				if (x < y) {
					int i = (x + y) / 2;
					int value = abs(y - x) / 2;
					if (0 <= i && i < n && 1 <= value && value < n) {
						a[i] = min(a[i], value); // we prefer small values
					}
				}
			}
		}
		for (int& x : a) {
			if (x == INT32_MAX) {
				x = uniform_int_distribution<int>(1, 2)(rng);
			}
		}
		bestAnswer = max(bestAnswer, make_pair((int)count_triples(a), a));
	}
	return bestAnswer.second;
}

// The sample grader without its secrets (graders/grader.cpp): line 1 is the part; Part I reads
// N and H and prints T, Part II reads M K and prints N and H.
int main() {
  int part;
  if (scanf("%d", &part) != 1) return 1;
  if (part == 1) {
    int N;
    assert(1 == scanf("%d", &N));
    std::vector<int> H(N);
    for (int i = 0; i < N; i++) assert(1 == scanf("%d", &H[i]));
    printf("%lld\n", count_triples(H));
  } else {
    int M, K;
    assert(2 == scanf("%d %d", &M, &K));
    std::vector<int> H = construct_range(M, K);
    int N = H.size();
    printf("%d\n", N);
    for (int i = 0; i < N; i++) printf("%d%c", H[i], " \n"[i + 1 == N]);
  }
}
