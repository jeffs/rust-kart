
# A main file in src:
$ cargo init my-project
$ cd my-project
$ cargo norm src/main.rs
my-project

# A program in a bin directory:
$ mkdir src/bin
$ touch src/bin/foo.rs
$ cargo norm src/bin/foo.rs
foo

# A main file in a subdirectory of bin:
$ mkdir src/bin/bar
$ touch src/bin/bar/main.rs
$ cargo norm src/bin/bar/main.rs
bar

# 


$
$ cd simple
$ 
$ cargo norm src/main.rs
simple

$ cd ..
$ cargo init cmd
