// plugin/src/lib.rs

use clap::{ArgMatches, Command, CommandFactory};
use polars::prelude::*;

pub mod contracts;

/// A catch-all Polars error.
pub type Result<T> = std::result::Result<T, PolarsError>;

/// Metadata about a plugin
/// Your plugin contract.  
/// * Drop the old `command()`; instead each plugin provides its own
///   `#[derive(clap::Args)]` type in `C`.
///
/// * `I` / `O` remain generic so you can later swap in other inputs/outputs.
/// * `run` is now async.
pub trait Plugin<I, O>: Send + Sync + 'static {
    type Config: clap::Parser + Sized + Send + Sync + Default + 'static;

    /// process `input` → `O`
    async fn run(&self, cfg: Self::Config, input: I) -> Result<O>;
}

/// One entry in the registry:  
/// - `name` for lookup  
/// - `parser()` to build the subcommand  
/// - `factory()` to get a `&'static dyn Plugin<I,O>`
pub struct PluginRegistration {
    pub name: &'static str,
    pub about: &'static str,
    pub parser: fn() -> Command,
    pub factory: fn() -> &'static str,
}

inventory::collect!(PluginRegistration);

/// iterate / lookup by `name`
pub fn iter() -> impl Iterator<Item = &'static PluginRegistration> {
    inventory::iter::<PluginRegistration>
        .into_iter()
        .collect::<Vec<_>>() // avoid borrow issues
        .into_iter()
}

pub fn lookup(name: &str) -> Option<&'static PluginRegistration> {
    iter().find(|reg| reg.name == name)
}

/// Glue into your top-level CLI:
/// ```rust
/// let app = attach_all_subcommands(app);
/// ```
pub fn attach_all_subcommands(app: Command) -> Command {
    iter().fold(app, |app, reg| app.subcommand((reg.parser)()))
}

fn temp_create_plugin() -> &'static str {
    "temp"
}

/// Dispatches into the right plugin (async).
pub async fn dispatch(matches: &ArgMatches, df: DataFrame) -> Result<Option<DataFrame>> {
    if let Some((sub, sub_m)) = matches.subcommand() {
        if let Some(reg) = lookup(sub) {
            let plugin = (reg.factory)();
            todo!()
            // let out = plugin.run(sub_m.clone(), df).await?;
            // return Ok(Some(out));
        }
    }
    Ok(None)
}

/// Macro to register a plugin `T: Plugin + Default`
///
/// Expands to:
/// 1) a single `static INSTANCE`  
/// 2) a `fn factory() -> &'static dyn Plugin`  
/// 3) a `fn parser()  -> Command` (via `T::Config::command()`)  
/// 4) `inventory::submit!` of a `PluginRegistration`
#[macro_export]
macro_rules! register_plugin {
    ($ty:ty, $cfg:ty) => {
        // your one and only instance
        static PLUGIN_INSTANCE: $ty = <$ty>::default();

        // factory fn pointer
        fn __factory() -> &'static dyn Plugin<DataFrame, DataFrame, ArgMatches> {
            &PLUGIN_INSTANCE
        }

        // build the clap::Command from your derive
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
