TODO

- Fix shutdown flow (double `wait_until_shutdown()`):
  - Remove the second wait inside `shut_down_task` and keep the drain + `shutdown::execute` after the first wait completes.
  - Add a brief comment/doc on the intended shutdown sequence to avoid reintroducing the double wait.
  - Verify graceful drain still sleeps 1s and runs shutdown logic once.
- Initialization spawn robustness:
  - Wrap `operations::initialization::init` in a spawned task that logs any `Err` and catches panics (log with `my_logger`).
  - Include enough context in the log (e.g., "init failed" with error and maybe settings snapshot/hostname).
  - Decide if a failure should trigger `states.shutdown` or keep running; document choice inline.
