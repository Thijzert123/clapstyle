use clapstyle::ResultExt;

fn main() -> std::process::ExitCode {
    try_main().report_clapstyle()
}

fn try_main() -> clapstyle::Result<()> {
    read_file()?;
    Ok(())
}

fn read_file() -> anyhow::Result<()> {
    std::fs::read_to_string("doesnt_exist")?;
    Ok(())
}
