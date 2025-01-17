## Rinha de Compiler

This is my implementatio of an interpreter for the [rinha de compilers](https://github.com/aripiprazole/rinha-de-compiler) competition.

Although the challenge's name includes 'compilers', the organizers made their own [compiler](https://docs.rs/rinha/latest/rinha/), which generates the AST. Since I'm starting from the AST, my solution is an interpreter.

It's my first time with an interpreter and I am new to Rust, so take everything here with a piece of salt.

## Build

```
docker build -t interpreter .
```

## Run

Map a file named `source.rinha.json` to the container:

```
docker run --rm -v ./source.rinha.json:/var/rinha/source.rinha.json interpreter
```


## TODO or ideas to improve

- [ ] - Use command line to turn on/off memoization (clap crate)
- [ ] - Memoize only pure functions
- [ ] - Memoize binary operations
- [ ] - Apply Tail Optimization
- [ ] - Test more scenarios
- [X] - Print closures
- [X] - Support `Tuple`
