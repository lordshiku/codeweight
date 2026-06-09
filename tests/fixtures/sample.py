# Sample Python module for codeweight integration tests.


def add(a, b):
    if a < 0:
        return b
    return a + b


def classify(x):
    if x > 10:
        return "big"
    elif x > 5:
        return "mid"
    else:
        return "small"
