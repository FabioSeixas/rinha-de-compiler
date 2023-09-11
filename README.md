## Rinha de Compiler

This is my implementatio of an interpreter for the [rinha de compilers](https://github.com/aripiprazole/rinha-de-compiler) competition.

Although the challenge's name includes 'compilers', the organizers made their own [compiler](https://docs.rs/rinha/latest/rinha/), which generates the AST. Since I'm starting from the AST, my solution is an interpreter.

It's my first time with an interpreter and I am new to Rust, so take everything here with a piece of salt.

## TODO

- [ ] - Use command line to turn on/off memoization (clap crate)
- [ ] - Fix memoization (`fib(50)` is returning a negative number 😅) -> `result > 2^(32-1)`. The fix is `Panic`.
- [ ] - Memoize only pure functions
- [ ] - Memoize binary operations
- [ ] - Apply Tail Optimization
- [ ] - Test more scenarios
- [ ] - Print closures
- [ ] - Support `Tuple`
