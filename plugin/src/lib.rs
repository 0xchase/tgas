use clap::{ArgMatches, Command, CommandFactory};
use polars::prelude::*;

pub mod contracts;

pub type Result<T> = std::result::Result<T, PolarsError>;

pub trait Plugin<I, O>: Send + Sync + 'static {
    type Config: clap::Parser + Sized + Send + Sync + Default + 'static;

    async fn run(&self, cfg: Self::Config, input: I) -> Result<O>;
}

pub struct PluginRegistration {
    pub name: &'static str,
    pub about: &'static str,
    pub parser: fn() -> Command,
    pub factory: fn() -> &'static str,
}

inventory::collect!(PluginRegistration);

pub fn iter() -> impl Iterator<Item = &'static PluginRegistration> {
    inventory::iter::<PluginRegistration>
        .into_iter()
        .collect::<Vec<_>>()
        .into_iter()
}

pub fn lookup(name: &str) -> Option<&'static PluginRegistration> {
    iter().find(|reg| reg.name == name)
}

pub fn attach_all_subcommands(app: Command) -> Command {
    iter().fold(app, |app, reg| app.subcommand((reg.parser)()))
}

fn temp_create_plugin() -> &'static str {
    "temp"
}

pub async fn dispatch(matches: &ArgMatches, df: DataFrame) -> Result<Option<DataFrame>> {
    if let Some((sub, sub_m)) = matches.subcommand() {
        if let Some(reg) = lookup(sub) {
            let plugin = (reg.factory)();
            todo!()

        }
    }
    Ok(None)
}

#[macro_export]
macro_rules! register_plugin {
    ($ty:ty, $cfg:ty) => {
        static PLUGIN_INSTANCE: $ty = <$ty>::default();

        fn __factory() -> &'static dyn Plugin<DataFrame, DataFrame, ArgMatches> {
            &PLUGIN_INSTANCE
        }

        fn __parser() -> Command {
            <$cfg as clap::CommandFactory>::command()
                .name(stringify!($ty))
                .about("Plugin configuration")
        }

        inventory::submit! {
            $crate::PluginRegistration {
                name:    stringify!($ty),
                about:   "Plugin configuration",
                parser:  __parser,
                factory: __factory,
            }
        }
    };
}
