# `hal`

> Hardware Abstraction Layer

## Highlights

### `hal::time::Instant`

A monotonically nondecreasing timer is configured and started before `main`.
`hal::time::Instant` inter-operates with `core::time::Duration`, which uses
human-friendly units of time (i.e. seconds).

This API is indispensable for any development as it lets the developer insert
timeouts when checking for changes in status flags. It's easy to get these
checks wrong when first reading the reference manual and without a timeout
mechanism one may end up using unbound `while` loops, which can leave the
program stuck in an infinite loop.
