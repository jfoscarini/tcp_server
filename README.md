# Live Server

A POC (proof of concept) tcp server to evaluate whether a migration of current live services to Rust would be a good idea.

There are no intended changes to be made to this POC. It has served its purpose.

## What is it

## How to run it

```console
$ ./server -p 8000
```

## Examples

### TCP Client

Bash
```console
$ telnet 127.0.0.1 8000
Hello World
```
