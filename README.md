# About
A simple parser to parse expressions with +, -, /, *, (, ), !, ~, ^ using pratt parsing
It takes in such an expression, converts it into a token stream (scanning),
parses it with a pratt parser, then outputs the result as an expression in a tree-like format
