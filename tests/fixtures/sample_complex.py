# Complex Python module for codeweight integration tests.


def simple_add(a, b):
    return a + b


def classify_nested(value, threshold):
    if value > threshold:
        if value > threshold * 2:
            return "high"
        return "mid"
    elif value < 0:
        return "negative"
    return "low"


def process_items(items):
    total = 0
    for item in items:
        if item > 0:
            total += item
        elif item < 0:
            total -= 1
    return total


def complex_handler(flag, data):
    if not flag:
        return None
    result = []
    for row in data:
        if row is None:
            continue
        if len(row) > 3:
            for cell in row:
                if cell > 10:
                    result.append(cell * 2)
                else:
                    result.append(cell)
        else:
            result.append(sum(row))
    return result


def risky_branch(x, y):
    if x and y:
        if x > y:
            return x
        elif x == y:
            return 0
    return y or x
