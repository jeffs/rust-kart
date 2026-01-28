# Tmux-send

The tmux-send program, when run in a tmux session, sends its standard input
to the bottom rightmost tmux pane.  This is useful when binding keys in a text
editor to send snippets of code to a live shell or REPL; e.g., in [Vim][vim]
or [Helix][hx].

In addition to tmux, this tool supports WezTerm and Zellij. Zellij's notion
of "next" pane is a little wacky when it comes to stacked panes: It goes in
_increasing_ Z order, which corresponds to upward vertical ordering; so if you
want to send text from a stacked pane on the left to a pane on the right, the
left pane must be at the top of the stack. Usually, "next" means down; but for
Zellij stacks, it means up.

[vim]: https://github.com/jeffs/geode-profile-home/blob/3b657a2f9b75916eef71202bf644ebce61022f2e/etc/nvim/init.vim#L82-L85

[hx]: https://github.com/jeffs/conf/blob/21c67133c5c77b10920d5fc15a33d553a6e34cfb/etc/helix/config.toml#L80
