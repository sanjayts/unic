# wcr (wc in Rust)
wc implemented in Rust as part of reading command line rust book. This repo uses the latest version of clap which has
quite a different API compared to the API used in the book.

This program supports the following capabilities:

```shell
unic 1.0.0
sanjayts

USAGE:
    unic [OPTIONS] [ARGS]

ARGS:
    <INPUT>     [default: -]
    <OUTPUT>    

OPTIONS:
    -c, --count      Prefix lines by the number of occurrences
    -h, --help       Print help information
    -V, --version    Print version information
```

# Challenges

* One of the main challenge was tackling whitespace 


# TODO

* Add support for `-d` or `--repeated`
* Add support for `-u` (lines which are not repeated)
* Add support for `--skip-fields` and `--skip-chars`

# Reference

* https://www.freebsd.org/cgi/man.cgi?query=uniq