# text-utils

Text transformation utilities.

## Binaries

- **rot13**: ROT13 cipher (also rotates digits by 5)
- **fix-quotes**: Replaces smart quotes with ASCII equivalents
- **hexify**: Converts decimal digit sequences to hexadecimal
- **hexwords**: Filters stdin for words representable in hex

## Usage

```sh
echo "Hello" | rot13
# Output: Uryyb

echo "Hello" | fix-quotes  # fixes curly quotes

hexify "192.168.1.1"
# Output: c0.a8.1.1

cat /usr/share/dict/words | hexwords | head
# Words like: cafe, face, bead, dead...
```
