[![asciicast](https://asciinema.org/a/zVWEplStTzXKpTJCpR1ixVWlX.svg)](https://asciinema.org/a/zVWEplStTzXKpTJCpR1ixVWlX)

A TUI (terminal user interface) for [Edinburgh University's Learn](https://www.learn.ed.ac.uk).

It provides a simple and fast interface for viewing course information and downloading files.
Submitting assignments and editing courses are explicitly not goals: They are complex and you probably shouldn't trust random programs with your university assignments.

This could be adapted to work with other systems using Blackboard Learn, but the supported services and authentication process are currently tied to UoE.

## Usage

To use, first install as normal using `cargo`. Currently only Linux is supported, but other systems should work.
Run with `edlearn_tui`.

## Developing

Development is split across several crates:

* `edlearn_client` - Rust wrapper around the web API
* `bbml` - A library for rendering a subset of HTML to be displayed in `ratatui` applications
* `edlearn_tui` - The main application

To learn more about the structure of each, check the rustdocs.

## License

Unless otherwise noted, content in this repository is licensed under the GNU GPL v3.0.
