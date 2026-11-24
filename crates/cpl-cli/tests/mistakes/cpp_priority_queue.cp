// here: 3
// C++'s `priority_queue` (self-run mistakes): no fix, then `pq.top()` was "fixed" to `pq.pop()`.
priority_queue<int> pq;
pq.push(1)
pq.push(3)
print(pq.top())
priority_queue<int, vector<int>, greater<int>> mn;
mn.push(5)
mn.push(2)
print(mn.top())
