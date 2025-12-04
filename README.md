## Debugger

Emulator has a debugger that you can run via cli `-d`, `--debug` or via a script `-f`, `--debug-file`.
Most values are assumed to be hex values

### Full list of debugger commends

```sh
# Comments start with '#'

# Use b or break to set break point on any address
b     08000188
break 08000188
# You can also use segment:offset notation
b     202b:002b

# p/print prints current state of Cpu
p
print
# You can also use print to print values from address or segment:offset
p 202b:002b

# You can turn on/off logging (verbose mode)
logon
logoff

# Stop executing the script
q
quit
exit

# run until next breakpoint, if any is found
r
run

# execute current instruction and go to next instruction
n
next

# While break loop will run commands while the emulator breaks on the same address
# multiple times in a row. When new address is found, execution will be stopped
while break 202b:002c {
    p 202b:002c
    p es:di
}
```
