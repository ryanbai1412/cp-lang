// IOI 2025 Day 1 "Souvenirs": the official model solution
// (https://github.com/ioi/task-archive/blob/master/2025/day1/souvenirs/solutions/model_solution/author_solution.cpp)
// with souvenirs.h inlined and a sample grader main that reads the port's input.
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

// Sample grader (IOI 2025 attachments, with the judge's 5000-call limit): N, then
// P[0] .. P[N-1]; prints how many souvenirs of each type were bought, or why it stopped.
static int grader_n, grader_calls;
static std::vector<long long> grader_p;
static std::vector<int> grader_q;

static void grader_quit(const char* message) {
    printf("%s\n", message);
    exit(0);
}

std::pair<std::vector<int>, long long> transaction(long long M) {
    if (++grader_calls > 5000) grader_quit("Too many calls");
    if (M >= grader_p[0] || M < grader_p[grader_n - 1]) grader_quit("Invalid argument");
    std::vector<int> L;
    long long R = M;
    for (int i = 0; i < grader_n; i++)
        if (R >= grader_p[i]) {
            R -= grader_p[i];
            grader_q[i]++;
            L.push_back(i);
        }
    return {L, R};
}

int main() {
    if (scanf("%d", &grader_n) != 1) return 1;
    grader_p.resize(grader_n);
    for (auto& x : grader_p) scanf("%lld", &x);
    grader_q.assign(grader_n, 0);
    buy_souvenirs(grader_n, grader_p[0]);
    for (int i = 0; i < grader_n; i++) printf("%s%d", i ? " " : "", grader_q[i]);
    printf("\n");
}
