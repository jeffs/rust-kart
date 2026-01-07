# csv2json

Converts CSV from stdin to JSON on stdout.

## Usage

```sh
echo -e "name,age\nAlice,30\nBob,25" | csv2json
```

Output:
```json
[
  {
    "name": "Alice",
    "age": "30"
  },
  {
    "name": "Bob",
    "age": "25"
  }
]
```
