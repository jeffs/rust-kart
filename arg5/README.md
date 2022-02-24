# Arg5

Arg5 is an argument-parsing library.  When you declare an argument with
arg5, you pass it a mutable reference to an existing variable.  When you
call the parse() function, arg5 assigns the parsed values to whichever
variables you specified.

Arg5 is very much a work in progress.  It currently supports only integers
and strings, but the approach seems solid.  It generates decent (if
minimal) error and usage messages.
