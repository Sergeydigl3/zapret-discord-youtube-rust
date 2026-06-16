use super::FirewallBackend;

macro_rules! define_backends {
    (
        $(
            $(#[$attr:meta])*
            $variant:ident => $module:ident : $struct:ident as $name:literal,
        )+
    ) => {
        #[derive(PartialEq, Clone, Copy, Debug)]
        pub enum LinuxBackend {
            $(
                $(#[$attr])*
                $variant,
            )+
        }

        impl LinuxBackend {
            pub fn to_string(&self) -> String {
                match self {
                    $(LinuxBackend::$variant => rust_i18n::t!($name).into_owned()),+
                }
            }

            pub fn variants() -> Vec<LinuxBackend> {
                let all = vec![$(LinuxBackend::$variant),+];
                all.into_iter().filter(|b| b.available()).collect()
            }

            pub fn available(&self) -> bool {
                match self {
                    $(LinuxBackend::$variant => $module::is_available()),+
                }
            }

            pub fn from_config(s: &str) -> Self {
                match s {
                    $(stringify!($module) => {
                        let v = LinuxBackend::$variant;
                        if v.available() { v } else { Self::default_available() }
                    }),+
                    _ => Self::default_available(),
                }
            }

            fn default_available() -> Self {
                let all = [$(LinuxBackend::$variant),+];
                all.into_iter().find(|b| b.available()).unwrap_or(all[0])
            }

            pub fn to_config(&self) -> &'static str {
                match self {
                    $(LinuxBackend::$variant => stringify!($module)),+
                }
            }
        }

        impl FirewallBackend for LinuxBackend {
            fn setup(&self, tcp_ports: &str, udp_ports: &str, interface: &str) -> Result<(), String> {
                match self {
                    $(LinuxBackend::$variant => $module::$struct.setup(tcp_ports, udp_ports, interface)),+
                }
            }

            fn clear(&self) -> Result<(), String> {
                match self {
                    $(LinuxBackend::$variant => $module::$struct.clear()),+
                }
            }
        }
    };
}

include!("_backends.rs");
