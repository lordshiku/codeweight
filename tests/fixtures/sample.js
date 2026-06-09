// Sample JavaScript module for codeweight integration tests.

function add(a, b) {
    if (a < 0) {
        return b;
    }
    return a + b;
}

function classify(x) {
    if (x > 10) {
        return "big";
    } else if (x > 5) {
        return "mid";
    } else {
        return "small";
    }
}

const identity = (value) => value;
