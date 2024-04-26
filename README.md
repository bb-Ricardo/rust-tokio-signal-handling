# Rust Tokio Signal handler example

This is a basic implementation of async signal handler for Rusts tokio async runtime.

following requirements should be covered:
* cross compiles to
  * Windows
  * Linux
  * Mac OSX
* don't interrupt write operations
* handle different kinds of signals
* shut down gracefully (and clean up) on termination
* implement a "double Ctrl+C" to give user the ability to force quit if shutdown takes to long
* usage of a shutdown timer if shutdown takes to long for unknown reasons

### Things to consider

From [Tasks, threads, and contention](https://tokio.rs/tokio/tutorial/shared-state)
> Using a blocking mutex to guard short critical sections is an acceptable strategy when contention is minimal. When a lock is contended, the thread executing the task must block and wait on the mutex. **This will not only block the current task but it will also block all other tasks scheduled on the current thread.**

So please make sure to use mutexs and locks wisely. Also beware of causing a [deadlock](https://en.wikipedia.org/wiki/Deadlock) situation

## List of resources:
* https://blog.logrocket.com/guide-signal-handling-rust
* https://rust-cli.github.io/book/in-depth/signals.html
* https://docs.rs/tokio/latest/tokio/signal/index.html
* https://tokio.rs/tokio/topics/shutdown
* https://stackoverflow.com/questions/77585473/rust-tokio-how-to-handle-more-signals-than-just-sigint-i-e-sigquit
* https://www.youtube.com/watch?v=zhbkp_Fzqoo

thanks to @andreasklostermaier for all the suggestions

# License
>You can check out the full license [here](https://github.com/bb-Ricardo/rust-tokio-signal-handling/blob/main/LICENSE.txt)

This project is licensed under the terms of the **MIT** license.