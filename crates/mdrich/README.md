# mdrich

Read Markdown on stdin, render it, and put it on the macOS pasteboard so
that Cmd+V pastes as rich text into Slack, Gmail, browsers, Notion,
Discord, and any other HTML-rendering destination.

The pasteboard is set with two UTIs:

- `public.html` — the rendered HTML
- `public.utf8-plain-text` — a plain-text fallback

`NSPasteboard` auto-derives the legacy aliases (`Apple HTML pasteboard
type`, `NSStringPboardType`), so after a run the type list resembles
what a browser produces on Cmd+C of formatted text.

## Why this binary exists

The previous incarnation was a shell script that piped through pandoc
and JXA. We replaced it with a single Rust binary because the chain ran
many times per day and the latency, dependency footprint, and shell
quoting hazards added up.

## Pitfalls already paid for

- `pbcopy` always lands data as `public.utf8-plain-text` regardless of
  what its man page claims about RTF/EPS sniffing, so it cannot be used
  to advertise HTML.
- `AppleScript`'s `«class RTF »` sets a legacy four-char-code that modern
  apps ignore for Cmd+V; UTIs are the only thing that matters.
- RTF is the wrong format here. Slack, Gmail, browsers, and Notion
  consume HTML, not RTF. RTF is for the TextEdit/Mail family, which
  also accept HTML, so HTML is the correct
  lowest-common-denominator.

## Usage

    echo '**bold**' | mdrich

Then Cmd+V into any rich-text destination.
