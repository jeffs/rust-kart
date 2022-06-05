Adjacency implies precedence.

Parsed subtrees may not be full.

    next_tree(&str) -> Tree
    anneal(Tree, Tree) -> Tree


# 6x 5+ 2

      x       +     2
     /       /
    6       5


     x
    / \
   6   +
      / \
     5   2


# 6x 5 + 2

      x     5   +   2
     /
    6

        +
       / \
      x   2
     / \
    6   5


# 6+ 5x 2

      +       x     2
     /       /
    6       5


     +
    / \
   6   x
      / \
     5   2


# 6+ 5 x 2

      +     5   x   2
     /
    6

     +
    / \
   6   x
      / \
     5   2


Because x has higher precedence than +.
