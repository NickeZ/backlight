# Introduction

Brightness controller

# Usage

```
backlight 0.1.0
Niklas Claesson <nicke.claesson@gmail.com>

USAGE:
    backlight [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -q, --quiet
    -V, --version    Prints version information

SUBCOMMANDS:
    dec
    get
    help    Prints this message or the help of the given subcommand(s)
    inc
    set
```

## I3 Config

```
bindsym XF86MonBrightnessDown exec --no-startup-id backlight dec 5
bindsym XF86MonBrightnessUp exec --no-startup-id backlight inc 5
```
