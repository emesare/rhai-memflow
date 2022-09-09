use std::cell::RefCell;

use clap::{Arg, ArgMatches, Command};
use memflow::prelude::v1::*;
use rhai::{packages::Package, Engine, Scope};
use rhai_memflow::{os::SharedOs, MemflowPackage};

fn main() -> Result<()> {
    let matches = parse_args();
    let chain = extract_args(&matches)?;

    // Create our inventory and OS.
    let inventory = Inventory::scan();
    let os = inventory.builder().os_chain(chain).build()?;

    // Register our memflow package.
    let mut engine = Engine::new();
    let package = MemflowPackage::new();
    package.register_into_engine(&mut engine);

    // Add our OS to rhai scope.
    let mut scope = Scope::new();
    let shared_os: SharedOs = RefCell::new(os);
    scope.push_constant("OS", shared_os);

    // Run `calculator.rhai` script.
    engine
        .eval_with_scope::<()>(&mut scope, include_str!("calculator.rhai"))
        .expect("eval failed");

    Ok(())
}

fn parse_args() -> ArgMatches {
    Command::new("calcaulator example")
        .arg(Arg::new("verbose").short('v').multiple_occurrences(true))
        .arg(
            Arg::new("connector")
                .long("connector")
                .short('c')
                .takes_value(true)
                .required(false)
                .multiple_values(true),
        )
        .arg(
            Arg::new("os")
                .long("os")
                .short('o')
                .takes_value(true)
                .required(true)
                .multiple_values(true),
        )
        .get_matches()
}

fn extract_args(matches: &ArgMatches) -> Result<OsChain<'_>> {
    let conn_iter = matches
        .indices_of("connector")
        .zip(matches.values_of("connector"))
        .map(|(a, b)| a.zip(b))
        .into_iter()
        .flatten();

    let os_iter = matches
        .indices_of("os")
        .zip(matches.values_of("os"))
        .map(|(a, b)| a.zip(b))
        .into_iter()
        .flatten();

    Ok(OsChain::new(conn_iter, os_iter)?)
}
