# Len

Len is a program to sort lines by length.  It was written to help give a feel
for de facto conventions in an unfamiliar codebase.

The implementation of Len includes a fault tolerant iterator, FilesLines, over
lines in specified files or directory trees.  FilesLines should be factored out
of len, as it is generally useful for implementing command-line tools.
