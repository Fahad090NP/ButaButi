# Proptest Regression Test Cases

This directory contains **automatically generated regression test cases** from the `proptest` property-based testing framework.

## What is this?

When proptest discovers a failing test case during randomized property testing, it:

1. Automatically saves the failing case to a `.txt` file in this directory
2. Re-runs that exact case on every subsequent test run
3. Ensures the bug doesn't reappear (regression testing)

## Why is this useful?

- **Prevents regressions**: Once a bug is found and fixed, the failing test case is permanently saved
- **Minimal test cases**: Proptest automatically shrinks failing cases to their simplest form
- **No manual work**: Test cases are generated and managed automatically
- **Reproducibility**: Exact same inputs are tested every time

## Current regression tests

### `core/pattern.txt`

- **Module**: `core::pattern`
- **Test**: Pattern splitting with large coordinates
- **Discovery**: Found edge case during property-based testing
- **Purpose**: Ensures pattern splitting handles extreme coordinate values correctly

## Should these files be committed?

**YES** - According to [proptest documentation](https://altsysrq.github.io/proptest-book/proptest/getting-started.html#persistence):

> "It is recommended to commit the `proptest-regressions` directory to version control. This allows the tests to reproduce failures across different machines and CI environments."

## How to use

1. **Run tests normally**: `cargo test --lib`
   - Proptest will automatically replay all saved regression cases
   - If a saved case fails, the test will fail (indicating a regression)

2. **Clear all regressions** (if needed):

   ```powershell
   Remove-Item -Recurse proptest-regressions/*
   ```

3. **View saved cases**:
   - Files are in human-readable format (text files)
   - Each line represents one test case that previously failed
   - Format: `cc <seed> <test_input_data>`

## Directory structure

```sh
proptest-regressions/
├── README.md           # This file
└── core/
    └── pattern.txt     # Regression cases for core::pattern tests
```

Each subdirectory corresponds to a module path in the codebase. Files are named after the test function that generated them.

## More information

- [Proptest Book - Persistence](https://altsysrq.github.io/proptest-book/proptest/getting-started.html#persistence)
- [Proptest Crate Documentation](https://docs.rs/proptest/)
