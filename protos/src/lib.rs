#![allow(dead_code,non_camel_case_types,unused_unsafe,unused_variables)]
#![allow(non_upper_case_globals,non_snake_case,unused_imports,missing_docs)]

#[macro_use]
extern crate wayland_sys;
extern crate wayland_protocols;
extern crate wayland_commons;
#[cfg(feature = "client")]
extern crate wayland_client;
#[cfg(feature = "server")]
extern crate wayland_server;
#[macro_use]
extern crate bitflags;


pub mod layer_shell {
    #[cfg(feature = "client")]
    pub use self::generated::client::c_api as client;
    #[cfg(feature = "server")]
    pub use self::generated::server::c_api as server;

    mod generated {
        #[cfg(feature = "client")]
        pub mod client {
            pub mod c_interfaces {
                pub(crate) use wayland_sys::common::{wl_argument, wl_interface};
                pub(crate) use wayland_client::sys::protocol_interfaces::{wl_surface_interface, wl_output_interface};
                pub(crate) use wayland_protocols::xdg_shell::c_interfaces::xdg_popup_interface;
                include!(concat!(env!("OUT_DIR"), "/layer-shell-unstable-v1-interfaces.rs"));
            }

            pub mod c_api {
                pub(crate) use wayland_sys as sys;
                pub(crate) use wayland_client::{NewProxy, Proxy};
                pub(crate) use wayland_commons::{AnonymousObject, Interface, MessageGroup};
                pub(crate) use wayland_client::protocol::{wl_surface, wl_output};
                pub(crate) use wayland_protocols::xdg_shell::client::xdg_popup;
                include!(concat!(env!("OUT_DIR"), "/layer-shell-unstable-v1-client.rs"));
            }
        }

        #[cfg(feature = "server")]
        pub mod server {
            pub mod c_interfaces {
                pub(crate) use wayland_sys::common::{wl_argument, wl_interface};
                pub(crate) use wayland_server::sys::protocol_interfaces::{wl_surface_interface, wl_output_interface};
                pub(crate) use wayland_protocols::xdg_shell::c_interfaces::xdg_popup_interface;
                include!(concat!(env!("OUT_DIR"), "/layer-shell-unstable-v1-interfaces.rs"));
            }

            pub mod c_api {
                pub(crate) use wayland_sys as sys;
                pub(crate) use wayland_server::{NewResource, Resource};
                pub(crate) use wayland_commons::{AnonymousObject, Interface, MessageGroup};
                pub(crate) use wayland_server::protocol::{wl_surface, wl_output};
                pub(crate) use wayland_protocols::xdg_shell::server::xdg_popup;
                include!(concat!(env!("OUT_DIR"), "/layer-shell-unstable-v1-server.rs"));
            }
        }
    }
}
