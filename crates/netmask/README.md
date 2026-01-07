# netmask

Calculates the narrowest netmask that's inclusive of all specified netmasks.

## Usage

```sh
netmask 10.0.1.0/24 10.0.2.0/24
# Output: 10.0.0.0/22

netmask 192.168.1.1 192.168.1.2
# Output: 192.168.1.0/30
```

Netmasks may be in CIDR notation, or literal IP addresses (implicitly /32).
