[back to README.md](README.md)

# Contributing

## Overview
This package is designed to work with Holochain's Conductor API interfaces.


## Development

### Environment

- Developed using rustc `1.54.0 (a178d0322 2021-07-26)`
- Enter `nix-shell` for other development environment dependencies.

### Building
This is a library, not a binary.  No build required


### Testing

To run all tests with logging
```
make test-debug
```

- `make test-unit-debug` - **Unit tests only**
- `make test-integration-debug` - **Integration tests only**

> **NOTE:** remove `-debug` to run tests without logging
