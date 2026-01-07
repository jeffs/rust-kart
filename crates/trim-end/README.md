# trim-end

Removes trailing whitespace from files in place, or from stdin to stdout.

## Usage

```sh
# Process files in place
trim-end file1.txt file2.txt

# Process stdin
cat file.txt | trim-end

# Verbose mode
trim-end -v *.rs

# Also collapse consecutive blank lines
trim-end -l file.txt
```

When processing files, also removes trailing blank lines and ensures a final newline.
