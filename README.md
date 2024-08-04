# SimplePush: A Rust client for simplepush.io

[![Build](https://github.com/jasonfagan/simplepush-rs/actions/workflows/build.yml/badge.svg)](https://github.com/jasonfagan/simplepush-rs/actions?query=workflow%3ABuild)
[![crates.io](https://img.shields.io/crates/d/simplepush-rs.svg)](https://crates.io/crates/simplepush-rs)
[![Documentation](https://docs.rs/simplepush-rs/badge.svg)](https://docs.rs/simplepush-rs/)
[![Crates.io](https://img.shields.io/crates/v/simplepush-rs?logo=rust)](https://crates.io/crates/simplepush-rs/)

A Rust client for the [SimplePush API](https://simplepush.io/api)

## Adding the client to your project

```bash
cargo add simplepush-rs
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