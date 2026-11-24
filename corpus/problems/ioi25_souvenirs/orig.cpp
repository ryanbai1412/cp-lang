// IOI 2025 Day 1 "Souvenirs": the official model solution
// (https://github.com/ioi/task-archive/blob/master/2025/day1/souvenirs/solutions/model_solution/author_solution.cpp)
// with souvenirs.h inlined and a stdio transaction for interactor.py.
#include <bits/stdc++.h>

void buy_souvenirs(int N, long long P0);

std::pair<std::vector<int>, long long> transaction(long long M);
//WARNING: not tested
#include<bits/stdc++.h>

using namespace std;

using ll = long long;

vector<int> counters;
vector<ll> lead_cost;
vector<vector<int>> lead_bought;

int buy_next_lead(int i) {
    ll S = lead_bought[i].size();
    ll P = lead_cost[i]/S;
    if (S == 1) P = P - 1;
    auto pt = transaction(P);
    for (int j : pt.first) counters[j]++;
    int l = pt.first[0];
    lead_cost[l] = P-pt.second; 
    lead_bought[l] = pt.first;
    return l;
}

void buy_souvenirs(int N, ll P_0) {
    N--;
    counters = vector<int>(N+1, 0);
    lead_cost = vector<ll>(N+1, -1);
    lead_bought = vector<vector<int>>(N+1);

    lead_cost[0] = P_0;
    lead_bought[0] = {0};

    int lead_index = 0;

    while (lead_index < N) {
        lead_index = buy_next_lead(lead_index);
    }

    int prices_index = N;

    while (prices_index > 0) {
        if (lead_cost[lead_index] != -1) {
            while ((int)lead_bought[lead_index].size() > 1 && lead_cost[lead_bought[lead_index].back()] != -1) {
                lead_cost[lead_index] -= lead_cost[lead_bought[lead_index].back()];
                lead_bought[lead_index].pop_back();
            }
            if (lead_index == prices_index) {
                prices_index--;
                lead_index--;
            }
            else {
                lead_index = buy_next_lead(lead_index);
            }
        }
        else lead_index--;
    }

    for (int i=0; i <= N; ++i) {
        for (int j=counters[i]; j < i; ++j) {
            transaction(lead_cost[i]);
        }
    }
}

// Stdio form of the official manager's protocol (interactor.py): read "N P[0]"; each
// transaction(M) prints "? M" and reads "k L[0] .. L[k-1] R"; "!" ends the run.
std::pair<std::vector<int>, long long> transaction(long long M) {
    printf("? %lld\n", M);
    fflush(stdout);
    int k;
    if (scanf("%d", &k) != 1) exit(1);
    std::vector<int> L(k);
    for (auto& x : L) scanf("%d", &x);
    long long R;
    scanf("%lld", &R);
    return {L, R};
}
int main() {
    int n;
    long long p0;
    if (scanf("%d %lld", &n, &p0) != 2) return 1;
    buy_souvenirs(n, p0);
    printf("!\n");
    fflush(stdout);
}
