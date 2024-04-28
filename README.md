# ssping

Command-line tool for testing connectivity of Shadowsocks server.

## Usage

### Basic

```shell
ssping ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpiYXJmb28hQDEyNy4wLjAuMTo4Mzg4
```

Output:

```text
PING www.google.com via 127.0.0.1.
Status 204 from www.google.com via 127.0.0.1: time=24.5ms
Status 204 from www.google.com via 127.0.0.1: time=33.9ms
Status 204 from www.google.com via 127.0.0.1: time=27.8ms
Status 204 from www.google.com via 127.0.0.1: time=29.0ms
Status 204 from www.google.com via 127.0.0.1: time=29.2ms
Status 204 from www.google.com via 127.0.0.1: time=28.0ms
^C--- ping statistics ---
6 attempted, 6 succeeded, 0 errors, time 5468ms
```

### Number of requests

By default, it continues to send requests until being interrupted by Ctrl-C.
The number of requests can be limited so that it ends automatically.

```shell
ssping -c 3 ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpiYXJmb28hQDEyNy4wLjAuMTo4Mzg4
```

Output:

```text
PING www.google.com via 127.0.0.1.
Status 204 from www.google.com via 127.0.0.1: time=25.7ms
Status 204 from www.google.com via 127.0.0.1: time=24.1ms
Status 204 from www.google.com via 127.0.0.1: time=24.2ms
--- ping statistics ---
3 attempted, 3 succeeded, 0 errors, time 2076ms
```

### Test URL

By default, it uses `http://www.google.com/generate_204` for testing.
This URL may be changed to others, for example

```shell
ssping -u http://netcheck.upsuper.org \
  ss://Y2hhY2hhMjAtaWV0Zi1wb2x5MTMwNTpiYXJmb28hQDEyNy4wLjAuMTo4Mzg4
```

Output:

```text
PING netcheck.upsuper.org via 127.0.0.1.
Status 200 from netcheck.upsuper.org via 127.0.0.1: time=621.2ms
Status 200 from netcheck.upsuper.org via 127.0.0.1: time=404.0ms
Status 200 from netcheck.upsuper.org via 127.0.0.1: time=485.7ms
Status 200 from netcheck.upsuper.org via 127.0.0.1: time=533.8ms
^C--- ping statistics ---
4 attempted, 4 succeeded, 0 errors, time 5292ms
```

Any URL can work as long as it normally returns a successful HTTP status code (200~299).

### Exit code

* If any request is successful, the program exits with code `0`.
* If all requests fail, it exits with code `1`.
* If there is any other error, it exits with code `2`.
