# Stacked Errors

This is a very WIP experimental errors crate with the purpose to provide better error debuggability in `async` call stacks (backtraces are basically useless in `async`). What we do is take advantage of `#[track_caller]` and create two stacks of `Location`s and errors via the helpful `MapAddErr` trait.

Please file a PR if there is some error type you would like to support directly in the enum or add some other improvement.
