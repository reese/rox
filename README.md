<div align="center">
<h1>Rox</h1>

<img src="./assets/geodude.png"  alt="Geodude"/>
</div>

## Building from source

The Rox compiler is built in Rust and can be built using Rust's `cargo` command line tool.
You can start the REPL using `cargo run`, or you can run an individual file using `cargo run path/to/file.rox`.

## Difference between Lox and Rox

Rox is based off of the Lox language introduced in Bob Nystrom's book "Crafting Interpreters."
However, I intend to change some of the design decisions bade in the book.

While Rox is still a dynamically typed scripting language, I've opted to eschew some of the idioms found in similar languages like Python and JavaScript.
The most notable change is that I've removed the usage of the `nil` type.
In the future, I hope to implement more complex type options, such as an `Option` type and type annotations (or even static typing), but for right now, I have chosen to remove `nil` entirely.

Similarly, I've also chosen to remove the concept of "truthy" and "falsey" types.
In languages like Python, it's common to see something like the following:

```python
my_list = []

# do some work that conditionally appends to the list

if my_list:
    # do stuff
```

However, this concept of "truthiness" can be ambiguous and cause subtle errors.
Instead, I've opted to only check for boolean types in situations like this.
This forces the user to decide _what_ they're checking for.
Is the list empty?
Are all of its properties of the same type?
Is _`my_list`_ still of the `List` type?