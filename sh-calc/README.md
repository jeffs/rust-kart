# sc

The sc command is an ergonomic command-line calculator.  It accepts expressions
as arguments, and prints the result.  The input expression language is
familiar, but accepts operators that need not be quoted to avoid shell
interpolation, and that don't require the shift key on most keyboards.  For
example, sc recognizes `x` to mean multiplication.

```sh
$ sc 3x5
15

$ sc 3 + 5 x 2
13
```

Each argument is a distinct expression, so you don't need parentheses (which
generally require quoting anyway) for grouping.

```sh
$ sc '3 + 5' x 2
16
```

Expressions not separated by operators are evaluated independently, and their
results printed separately.

```sh
$ sc 1+2 3x4
3 12
```

Commas may be used as separators:

```sh
$ sc 1,000,000x2
2000000
```

The -c flag inserts separators in the output:

```sh
$ sc -c 1000000
1,000,000
```

Unlike an interactive prompt, sc does not require you to exit anything once
you've gotten what you came for.  If you're doing a lot of arithmetic, a REPL
like Python or Node is probably superior; but sc aims to be the quickest,
easiest way to evaluate simple arithmetic expressions.

Relative to other command-line calculators or REPLs like Python and Node, sc saves you 
Of course, sc recognizes

On most keyboards, using x instead of `*` and `''` instead of `()` saves you
from having to hold the Shift key, making expressions quicker and easier to
type.

# Prior Art

* Apple Spotlight supports x for multiplication, but requires whitespace around it
  - "26.3 x 17" works, but "26.3x17" does not
* Apple Spotlight 

# TODO

Discuss overflow.

[vim]: https://github.com/jeffs/geode-profile-home/blob/3b657a2f9b75916eef71202bf644ebce61022f2e/etc/nvim/after/ftplugin/rust.vim#L23-L29
