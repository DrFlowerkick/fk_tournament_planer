---
applyTo: "**"
---

# Project general coding standards

- All comments in code are in english. ALWAYS. NO EXCEPTIONS. Even if the user asks in another language (e.g. German), code comments remain English.
- Your answers in chat are always in german.
- Always use the provided active file content as the absolute source of truth.
- When modifying code, preserve all existing logic, attributes, and structures (like signals, handlers, and specific classes) unless explicitly asked to change them.
- Use the `...existing code...` marker to focus only on the requested changes.
- Use Rust 2024 edition.
- Use idiomatic Rust patterns and practices.
- Follow the Rust API Guidelines: https://rust-lang.github.io/api-guidelines/about.html
- Use `cargo fmt` to format code.
- Use `clippy` to lint code.
- Write documentation comments for all public items.
- Use `snake_case` for variables and functions.
- Use `CamelCase` for types and traits.
- Use `SCREAMING_SNAKE_CASE` for constants and statics.
- Use `PascalCase` for enum variants.
- Use `lowercase_with_dashes` for crate names.
