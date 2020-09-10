import math
import random
import time

def now():
    return time.time() * 1000

def format_time(ms):
    if ms < 1000: return "{}ms".format(ms)
    if ms < 60000: return "{}s".format(ms / 1000.0)
    minutes = math.floor(ms / 60000)
    remaining = ms - (minutes * 60000)
    return "{}min {}s".format(minutes, remaining / 1000.0)

def square(a):
    return a * a

def mx_plus_b(m, x, b):
    return m * x + b

def e_field(i, arr):
    l = len(arr)
    out = 0.0
    for n in range(0,l):
        distance = i - n
        if distance != 0:
            sqdistance = distance * distance
            invsqdistance = 1.0 / sqdistance
            scaled = invsqdistance * arr[n]
            out = out + scaled
    return out

def gen_rand_array(size):
    return [math.floor(random.random() * 100000.0) for _ in range(0,size)]

def lin_square(size):
    data = gen_rand_array(size)
    start = now()
    output = list(map(square, data))
    end = now()
    return end - start

def lin_mx_plus_b(size):
    m = 2
    b = 3
    data = gen_rand_array(size)
    start = now()
    output = list(map(lambda x: mx_plus_b(m, x, b), data))
    end = now()
    return end - start

def lin_e_field(size):
    data = gen_rand_array(size)
    start = now()
    output = list(map(lambda i: e_field(i, data), range(0, size)))
    end = now()
    return end - start

def benchmark():
    print("Python Benchmark!")
    print('Squares 100-element array: {}'.format(format_time(lin_square(100))))
    print('Squares 10,000-element array: {}'.format(format_time(lin_square(10000))))
    print('Squares 1,000,000-element array: {}'.format(format_time(lin_square(1000000))))
    print('mx+b 100-element array: {}'.format(format_time(lin_mx_plus_b(100))))
    print('mx+b 10,000-element array: {}'.format(format_time(lin_mx_plus_b(10000))))
    print('mx+b 1,000,000-element array: {}'.format(format_time(lin_mx_plus_b(1000000))))
    print('e-field 100-element array: {}'.format(format_time(lin_e_field(100))))
    print('e-field 10,000-element array: {}'.format(format_time(lin_e_field(10000))))
    #print('e-field 1,000,000-element array: {}'.format(format_time(lin_e_field(1000000))))

benchmark()