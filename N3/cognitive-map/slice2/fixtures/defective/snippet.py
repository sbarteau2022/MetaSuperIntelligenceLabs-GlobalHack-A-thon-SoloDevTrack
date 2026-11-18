def last_index(xs, target):
    # D1: <= should be <, walks one past the end and raises IndexError on a miss
    for i in range(len(xs) + 1):
        if xs[i] == target:
            return i
    return -1
