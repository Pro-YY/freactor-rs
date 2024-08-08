# Freactor
![crates.io](https://img.shields.io/crates/v/freactor)
![docs.rs](https://docs.rs/freactor/badge.svg)
![license](https://img.shields.io/crates/l/freactor)

A lightweight framework for asynchronous execution flow in Rust, designed to be fast, reliable, scalable and easy-to-use.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Usage](#usage)
- [Examples](#examples)
- [API Reference](#api-reference)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)

## Overview

Freactor is a lightweight and flexible framework designed to manage and execute asynchronous flows in Rust. It provides an easy-to-use API to define and execute asynchronous tasks, making it ideal for building complex workflows and state machines.

## Installation

Add with cargo command:
```
cargo add freactor
```

Or you can add `freactor` to your `Cargo.toml`:
```toml
[dependencies]
freactor = "0.1"
```

## Usage

Here's a quick example to get you started:
```
/* Your async function implementations, and function map here. */
async r1() {}
async r2() {}
async r3() {}
...


async run () {
    // 1. Define business flow config 
    let flow_config = r#"
    {
        "ExampleTask1": {
            "init_step": "r1",
            "config": {
                "r1": { "edges": ["r2", "r3", "r4"]},
                "r2": { "edges": ["r5", "r3"]},
                "r3": { "edges": ["r6", "r4"]},
                "r4": { "edges": []},
                "r5": { "edges": ["r6", "r3"]},
                "r6": { "edges": [], "retry": null}
            }
        }
    }
    "#.to_string();

    // 2. Init freactor with funcs and config    
    let f = Freactor::new(func_map, flow_config);

    // 3. Prepare you workspace arc state, and run your flow later anywhere
    let state = Arc::new(Mutex::new(State:new(YOUR_BUSINESS_DATA)))
    f.run("ExampleTask1", workspace_state).await;

}
```

## Examples

Here are some examples to illustrate how to use freactor for different scenarios:

**Example 1: Multi-Threaded Parallel Execution**
```
/* To be continued */
```

**Example 2: HTTP Web Server Integration**
```
/* To be continued */
```


## API Reference

For detailed API reference, please visit [docs.rs](https://docs.rs/freactor/latest/freactor/).

## Contributing

Welcome contributions from the community! Please read our [CONTRIBUTING](CONTRIBUTING.md) guide to learn how you can help.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

If you have any questions, feel free to reach out:

- GitHub: https://github.com/Pro-YY
- Email: paulyyangyang@gmail.com
