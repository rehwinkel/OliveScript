def sqrt(n, iters):
    a = 1.0
    i = 0
    while i < iters:
        a = 0.5 * (a + n / a)
        i = i + 1
    return a


i = 0
while i < 1000:
    sqrt(2, 1000)
    i = i + 1
