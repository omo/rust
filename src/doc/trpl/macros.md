% Macros

By now you've learned about many of the tools Rust provides for abstracting and
reusing code. These units of code reuse have a rich semantic structure. For
example, functions have a type signature, type parameters have trait bounds,
and overloaded functions must belong to a particular trait.

This structure means that Rust's core abstractions have powerful compile-time
correctness checking. But this comes at the price of reduced flexibility. If
you visually identify a pattern of repeated code, you may find it's difficult
or cumbersome to express that pattern as a generic function, a trait, or
anything else within Rust's semantics.

Macros allow us to abstract at a *syntactic* level. A macro invocation is
shorthand for an "expanded" syntactic form. This expansion happens early in
compilation, before any static checking. As a result, macros can capture many
patterns of code reuse that Rust's core abstractions cannot.

The drawback is that macro-based code can be harder to understand, because
fewer of the built-in rules apply. Like an ordinary function, a well-behaved
macro can be used without understanding its implementation. However, it can be
difficult to design a well-behaved macro!  Additionally, compiler errors in
macro code are harder to interpret, because they describe problems in the
expanded code, not the source-level form that developers use.

These drawbacks make macros something of a "feature of last resort". That's not
to say that macros are bad; they are part of Rust because sometimes they're
needed for truly concise, well-abstracted code. Just keep this tradeoff in
mind.

# Defining a macro

You may have seen the `vec!` macro, used to initialize a [vector][] with any
number of elements.

[vector]: arrays-vectors-and-slices.html

```rust
let x: Vec<u32> = vec![1, 2, 3];
# assert_eq!(x, [1, 2, 3]);
```

This can't be an ordinary function, because it takes any number of arguments.
But we can imagine it as syntactic shorthand for

```rust
let x: Vec<u32> = {
    let mut temp_vec = Vec::new();
    temp_vec.push(1);
    temp_vec.push(2);
    temp_vec.push(3);
    temp_vec
};
# assert_eq!(x, [1, 2, 3]);
```

We can implement this shorthand, using a macro: [^actual]

[^actual]: The actual definition of `vec!` in libcollections differs from the
           one presented here, for reasons of efficiency and reusability. Some
           of these are mentioned in the [advanced macros chapter][].

```rust
macro_rules! vec {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}
# fn main() {
#     assert_eq!(vec![1,2,3], [1, 2, 3]);
# }
```

Whoa, that's a lot of new syntax! Let's break it down.

```ignore
macro_rules! vec { ... }
```

This says we're defining a macro named `vec`, much as `fn vec` would define a
function named `vec`. In prose, we informally write a macro's name with an
exclamation point, e.g. `vec!`. The exclamation point is part of the invocation
syntax and serves to distinguish a macro from an ordinary function.

## Matching

The macro is defined through a series of *rules*, which are pattern-matching
cases. Above, we had

```ignore
( $( $x:expr ),* ) => { ... };
```

This is like a `match` expression arm, but the matching happens on Rust syntax
trees, at compile time. The semicolon is optional on the last (here, only)
case. The "pattern" on the left-hand side of `=>` is known as a *matcher*.
These have [their own little grammar] within the language.

[their own little grammar]: ../reference.html#macros

The matcher `$x:expr` will match any Rust expression, binding that syntax tree
to the *metavariable* `$x`. The identifier `expr` is a *fragment specifier*;
the full possibilities are enumerated in the [advanced macros chapter][].
Surrounding the matcher with `$(...),*` will match zero or more expressions,
separated by commas.

Aside from the special matcher syntax, any Rust tokens that appear in a matcher
must match exactly. For example,

```rust
macro_rules! foo {
    (x => $e:expr) => (println!("mode X: {}", $e));
    (y => $e:expr) => (println!("mode Y: {}", $e));
}

fn main() {
    foo!(y => 3);
}
```

will print

```text
mode Y: 3
```

With

```rust,ignore
foo!(z => 3);
```

we get the compiler error

```text
error: no rules expected the token `z`
```

## Expansion

The right-hand side of a macro rule is ordinary Rust syntax, for the most part.
But we can splice in bits of syntax captured by the matcher. From the original
example:

```ignore
$(
    temp_vec.push($x);
)*
```

Each matched expression `$x` will produce a single `push` statement in the
macro expansion. The repetition in the expansion proceeds in "lockstep" with
repetition in the matcher (more on this in a moment).

Because `$x` was already declared as matching an expression, we don't repeat
`:expr` on the right-hand side. Also, we don't include a separating comma as
part of the repetition operator. Instead, we have a terminating semicolon
within the repeated block.

Another detail: the `vec!` macro has *two* pairs of braces on the right-hand
side. They are often combined like so:

```ignore
macro_rules! foo {
    () => {{
        ...
    }}
}
```

The outer braces are part of the syntax of `macro_rules!`. In fact, you can use
`()` or `[]` instead. They simply delimit the right-hand side as a whole.

The inner braces are part of the expanded syntax. Remember, the `vec!` macro is
used in an expression context. To write an expression with multiple statements,
including `let`-bindings, we use a block. If your macro expands to a single
expression, you don't need this extra layer of braces.

Note that we never *declared* that the macro produces an expression. In fact,
this is not determined until we use the macro as an expression. With care, you
can write a macro whose expansion works in several contexts. For example,
shorthand for a data type could be valid as either an expression or a pattern.

## Repetition

The repetition operator follows two principal rules:

1. `$(...)*` walks through one "layer" of repetitions, for all of the `$name`s
   it contains, in lockstep, and
2. each `$name` must be under at least as many `$(...)*`s as it was matched
   against. If it is under more, it'll be duplicated, as appropriate.

This baroque macro illustrates the duplication of variables from outer
repetition levels.

```rust
macro_rules! o_O {
    (
        $(
            $x:expr; [ $( $y:expr ),* ]
        );*
    ) => {
        &[ $($( $x + $y ),*),* ]
    }
}

fn main() {
    let a: &[i32]
        = o_O!(10; [1, 2, 3];
               20; [4, 5, 6]);

    assert_eq!(a, [11, 12, 13, 24, 25, 26]);
}
```

That's most of the matcher syntax. These examples use `$(...)*`, which is a
"zero or more" match. Alternatively you can write `$(...)+` for a "one or
more" match. Both forms optionally include a separator, which can be any token
except `+` or `*`.

This system is based on
"[Macro-by-Example](http://www.cs.indiana.edu/ftp/techreports/TR206.pdf)"
(PDF link).

# Hygiene

Some languages implement macros using simple text substitution, which leads to
various problems. For example, this C program prints `13` instead of the
expected `25`.

```text
#define FIVE_TIMES(x) 5 * x

int main() {
    printf("%d\n", FIVE_TIMES(2 + 3));
    return 0;
}
```

After expansion we have `5 * 2 + 3`, and multiplication has greater precedence
than addition. If you've used C macros a lot, you probably know the standard
idioms for avoiding this problem, as well as five or six others. In Rust, we
don't have to worry about it.

```rust
macro_rules! five_times {
    ($x:expr) => (5 * $x);
}

fn main() {
    assert_eq!(25, five_times!(2 + 3));
}
```

The metavariable `$x` is parsed as a single expression node, and keeps its
place in the syntax tree even after substitution.

Another common problem in macro systems is *variable capture*. Here's a C
macro, using [a GNU C extension] to emulate Rust's expression blocks.

[a GNU C extension]: https://gcc.gnu.org/onlinedocs/gcc/Statement-Exprs.html

```text
#define LOG(msg) ({ \
    int state = get_log_state(); \
    if (state > 0) { \
        printf("log(%d): %s\n", state, msg); \
    } \
})
```

Here's a simple use case that goes terribly wrong:

```text
const char *state = "reticulating splines";
LOG(state)
```

This expands to

```text
const char *state = "reticulating splines";
int state = get_log_state();
if (state > 0) {
    printf("log(%d): %s\n", state, state);
}
```

The second variable named `state` shadows the first one.  This is a problem
because the print statement should refer to both of them.

The equivalent Rust macro has the desired behavior.

```rust
# fn get_log_state() -> i32 { 3 }
macro_rules! log {
    ($msg:expr) => {{
        let state: i32 = get_log_state();
        if state > 0 {
            println!("log({}): {}", state, $msg);
        }
    }};
}

fn main() {
    let state: &str = "reticulating splines";
    log!(state);
}
```

This works because Rust has a [hygienic macro system][]. Each macro expansion
happens in a distinct *syntax context*, and each variable is tagged with the
syntax context where it was introduced. It's as though the variable `state`
inside `main` is painted a different "color" from the variable `state` inside
the macro, and therefore they don't conflict.

[hygienic macro system]: http://en.wikipedia.org/wiki/Hygienic_macro

This also restricts the ability of macros to introduce new bindings at the
invocation site. Code such as the following will not work:

```rust,ignore
macro_rules! foo {
    () => (let x = 3);
}

fn main() {
    foo!();
    println!("{}", x);
}
```

Instead you need to pass the variable name into the invocation, so it's tagged
with the right syntax context.

```rust
macro_rules! foo {
    ($v:ident) => (let $v = 3);
}

fn main() {
    foo!(x);
    println!("{}", x);
}
```

This holds for `let` bindings and loop labels, but not for [items][].
So the following code does compile:

```rust
macro_rules! foo {
    () => (fn x() { });
}

fn main() {
    foo!();
    x();
}
```

[items]: ../reference.html#items

# Recursive macros

A macro's expansion can include more macro invocations, including invocations
of the very same macro being expanded.  These recursive macros are useful for
processing tree-structured input, as illustrated by this (simplistic) HTML
shorthand:

```rust
# #![allow(unused_must_use)]
macro_rules! write_html {
    ($w:expr, ) => (());

    ($w:expr, $e:tt) => (write!($w, "{}", $e));

    ($w:expr, $tag:ident [ $($inner:tt)* ] $($rest:tt)*) => {{
        write!($w, "<{}>", stringify!($tag));
        write_html!($w, $($inner)*);
        write!($w, "</{}>", stringify!($tag));
        write_html!($w, $($rest)*);
    }};
}

fn main() {
#   // FIXME(#21826)
    use std::fmt::Write;
    let mut out = String::new();

    write_html!(&mut out,
        html[
            head[title["Macros guide"]]
            body[h1["Macros are the best!"]]
        ]);

    assert_eq!(out,
        "<html><head><title>Macros guide</title></head>\
         <body><h1>Macros are the best!</h1></body></html>");
}
```

# Debugging macro code

To see the results of expanding macros, run `rustc --pretty expanded`. The
output represents a whole crate, so you can also feed it back in to `rustc`,
which will sometimes produce better error messages than the original
compilation. Note that the `--pretty expanded` output may have a different
meaning if multiple variables of the same name (but different syntax contexts)
are in play in the same scope. In this case `--pretty expanded,hygiene` will
tell you about the syntax contexts.

`rustc` provides two syntax extensions that help with macro debugging. For now,
they are unstable and require feature gates.

* `log_syntax!(...)` will print its arguments to standard output, at compile
  time, and "expand" to nothing.

* `trace_macros!(true)` will enable a compiler message every time a macro is
  expanded. Use `trace_macros!(false)` later in expansion to turn it off.

# Further reading

The [advanced macros chapter][] goes into more detail about macro syntax. It
also describes how to share macros between different modules or crates.

[advanced macros chapter]: advanced-macros.html
