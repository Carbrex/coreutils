Throw error if unset argument starts with '='

Before:
```
$ cargo run env | grep ZZZ 
ZZZ=zzz
$ cargo run env -u=ZZZ | grep ZZZ

(No output because ZZZ was unset)
```

After:
```
$ cargo run env | grep ZZZ
ZZZ=zzz
$ cargo run env -u=ZZZ | grep ZZZ
env: cannot unset '=ZZZ': Invalid argument
Try 'target/debug/coreutils env --help' for more information.
```

Tests:

Before:
```
$ cargo test test_env_with_gnu_reference_unset_invalid_variables

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 70 filtered out; finished in 0.00s


running 1 test
test test_env::test_env_with_gnu_reference_unset_invalid_variables ... FAILED

failures:

---- test_env::test_env_with_gnu_reference_unset_invalid_variables stdout ----
run: /home/geet/debian-gsoc/coreutils/target/debug/coreutils env -u=2EKt
thread 'test_env::test_env_with_gnu_reference_unset_invalid_variables' panicked at tests/by-util/test_env.rs:578:10:
Command was expected to fail.
stdout = LC_ALL=C
TZ=UTC

 stderr = 
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    test_env::test_env_with_gnu_reference_unset_invalid_variables

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2875 filtered out; finished in 0.00s
```

After:
```
$ cargo test test_env_with_gnu_reference_unset_invalid_variables

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 70 filtered out; finished in 0.00s


running 1 test
test test_env::test_env_with_gnu_reference_unset_invalid_variables ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2875 filtered out; finished in 0.01s
```