# FD leak

The cargo.toml has 2 `bin` sections that compiles to `server` and `client`. 

## server

server accepts argument `--worker-threads`. If set to 0, it will run in current thread. If set to > 0, it will create worker threads.

This listen for incoming events at `/var/run/sock.sock` (hard coded)

## client

The client runs an infinite loop and each iteration spins up a task that's run in parallel. The task connects to `/var/run/sock.sock`, send a request, reads what was received and prints it.

### outcome

In a linux OS, when `server` is run with (`-worker-threads=0`) and once the open file limit is reached, they aren't freed and the `server` doesn't recover. It keeps printing the following line, logged when the `listener.accept()` fails.

```
[2022-06-21T18:26:43Z ERROR server] error accepting connection, error: Too many open files (os error 24)
```

The moment it's switched to a threadpool (`-worker-threads=1`), the behaviour changes and the FDs are freed even if it hits the open file limit.
