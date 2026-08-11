use anyhow::Context;
use std::fs;

// Use this crates' println macro, so that styles get piped through anstreams println.
// This makes sure the style codes get removed when the user can't see it (e.g. piping into a file)
use clapstyle::println;

fn main() {
    // Normal display, single line with last error
    if let Err(err) = try_main() {
        println!("{}", err);
    }
    println!();

    // Alternative display, single line with last error + one cause
    if let Err(err) = try_main() {
        println!("{:#}", err);
    }
    println!();

    // Full caused stack, multiple lines
    if let Err(err) = try_main() {
        println!("{:?}", err);
    }
    println!();

    // Original Debug formatting for Error
    if let Err(err) = try_main() {
        println!("{:#?}", err);
    }
}

fn try_main() -> clapstyle::Result<()> {
    process_file()?;
    Ok(())
}

fn process_file() -> anyhow::Result<()> {
    read_file().with_context(|| "Couldn't process file")
}

fn read_file() -> anyhow::Result<()> {
    fs::read_to_string("doesnt_exist").with_context(|| "Couldn't open file")?;
    Ok(())
}
