# rs-lambda
so i've started messing around with rust, and so far it seems pretty cool. anyway, i thought for my first actual program (besides some rudimentary feature tests), it could be cool to write a lambda calculus interpreter.

so far i've only written a lexer and a parser, and I'm not sure how idiomatic the code is. i'm having fun though.

## usage
```rs
$ cargo build --release
$ echo "(λx. λf. f x) ((λa. a) (λb. b)) (λb. λc. b)" | ./target/release/rs-lambda
Application {
    function: Application {
        function: Abstraction {
            bound_variable: "x",
            return_term: Abstraction {
                bound_variable: "f",
                return_term: Application {
                    function: Variable(
                        "f",
                    ),
                    argument: Variable(
                        "x",
                    ),
                },
            },
        },
        argument: Application {
            function: Abstraction {
                bound_variable: "a",
                return_term: Variable(
                    "a",
                ),
            },
            argument: Abstraction {
                bound_variable: "b",
                return_term: Variable(
                    "b",
                ),
            },
        },
    },
    argument: Abstraction {
        bound_variable: "b",
        return_term: Abstraction {
            bound_variable: "c",
            return_term: Variable(
                "b",
            ),
        },
    },
}
```
