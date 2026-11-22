var mq = MonoDeque<Int>(min)
mq.push(at: 0, val: 3)
let a = mq.best!
mq.push(at: 1, val: 5)
mq.push(at: 2, val: 2)
let b = mq.best!
mq.push(at: 3, val: 5)
mq.drop_before(min_at: 2)
let c = mq.best!
mq.drop_before(min_at: 3)
let d = mq.best!
mq.clear()
print(a, b, c, d, mq.best, none: "none")
