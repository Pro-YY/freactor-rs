# Freactor

[![Crates.io](https://img.shields.io/crates/v/freactor?logo=rust)](https://crates.io/crates/freactor)
[![Docs.rs](https://img.shields.io/badge/docs.rs-docs-blue?logo=docs.rs)](https://docs.rs/freactor/latest/freactor/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/Pro-YY/freactor-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Pro-YY/freactor-rs/actions/workflows/ci.yml)

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
// run with independent self data(state)

async fn run() {
    // function map and flow config here...
    let f = Freactor::new(func_map, flow_config);

    // multiple flow instance concurrently
    let mut shared_vecs: Vec<Arc<Mutex<State>>> = Vec::with_capacity(10);
    for i in 0..shared_vecs.capacity() {
        let state = State::new(...);
        shared_vecs.push(Arc::new(Mutex::new(state)));
    }

    let mut jset = JoinSet::new();
    for v in shared_vecs.clone() {
        let fc = f.clone();
        jset.spawn(async move {
            let _ = fc.run("Task1", v).await;
        });
    }
    while let Some(_res) = jset.join_next().await {}

    for v in shared_vecs {
        let vec = v.lock().unwrap();
        info!("Mutated Vec: {:?}", *vec);
    }
}
```

**Example 2: HTTP Web Server Integration**
```
// with web framework, like Axum
// just put freactor in shared server state (Extension/State) and run your task in handler

async fn handle_task_1(Extension(f): Extension<Arc<Freactor>>) -> &'static str {
    let v = Arc::new(Mutex::new(State::new(...)));
    let _ = f.run("Task1", v).await;
    "Hello, World!"
}

async fn main() {
    // function map and flow config here...
    let f = Freactor::new(func_map, flow_config);
    let shared_server_state = Arc::new(f);

    let app = Router::new()
    .route("/", get(root))
    .route("/1", get(handle_task_1))
    .route("/2", get(handle_task_2))
    .layer(Extension(shared_server_state));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
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
