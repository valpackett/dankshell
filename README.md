
# dankshell

dankshell is eventually going to be a lightweight yet powerful Wayland-based desktop environment.

dankshell is written in [Rust] and uses [Weston]/[weston-rs] for the compositor and [GTK]/[gtk-rs]/[relm] for the UI.

[Rust]: https://www.rust-lang.org
[Weston]: https://cgit.freedesktop.org/wayland/weston/
[weston-rs]: https://github.com/myfreeweb/weston-rs
[GTK]: https://www.gtk.org
[gtk-rs]: https://gtk-rs.org
[relm]: https://github.com/antoyo/relm

## Current status

Currently, it's an early proof-of-concept.
A GTK bar renders on the UI layer of the libweston compositor, using the `layer-shell` protocol's `get_layer_surface` message.

![Screenshot](https://unrelentingtech.s3.dualstack.eu-west-1.amazonaws.com/dankshell-prototype-1.png)

## Development

- [Get Nightly Rust](https://rustup.rs)
- Clone this repo and git submodules, e.g. `git submodule update --init --recursive`
- Use `make run` to run in development

## Contributing

This is currently a personal experiment, I'm not looking for contributions just yet.

## License

dankshell is available under the MIT License.  
For more information, please refer to the `COPYING` file.
