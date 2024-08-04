# SimplePush: A Rust client for simplepush.io

[![Rust](https://github.com/jasonfagan/simplepush/workflows/Rust/badge.svg)](https://github.com/jasonfagan/simplepush/actions?query=workflow%3ARelease)
[![crates.io](https://img.shields.io/crates/d/simplepush.svg)](https://crates.io/crates/simplepush)
[![Documentation](https://docs.rs/simplepush/badge.svg)](https://docs.rs/simplepush/)
[![Crates.io](https://img.shields.io/crates/v/simplepush?logo=rust)](https://crates.io/crates/simeplepush/)

A Rust client for the [SimplePush API](https://simplepush.io/api)

## Adding the client to your project

```bash
cargo add simplepush
```

## Sending a simple notification

```rust
   let result = SimplePush::send(Message::new(
        "SIMPLE_PUSH_KEY",
        Some("title"),
        "test message",
        None,
        None,
    ));
```

## Sending a simple notification with encryption

```rust
   let result = SimplePush::send(Message::new(
        "SIMPLE_PUSH_KEY",
        Some("title"),
        "test message",
        None,
        None,
        "ENCRYPTION_KEY",
        Some("SALT"),
    ));
```