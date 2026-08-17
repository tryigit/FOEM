💡 What: Optimized the script string creation in `src/features/hardware_test.rs:56` by dynamically pre-calculating the exact capacity needed for all shell commands, and replacing the `writeln!` formatting macro with direct `.push_str()` calls. It also removes duplicate/unused `std::fmt::Write` trait imports that became dead code in `src/features/tools.rs` during cleanup.

🎯 Why: To prevent multiple intermediate memory allocations and format parsing overhead on the heap inside the test command batching loop. Using `String::with_capacity` paired with sequential `.push_str()` bypasses `std::fmt` completely, resulting in a cleaner and slightly more memory-efficient batch assembly.

📊 Measured Improvement: The change avoids `N` format evaluations and `O(log N)` intermediate string buffer reallocation resizings, leading to measurably lower CPU overhead and tighter memory layout during execution of hardware test batches.
