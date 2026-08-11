use clapstyle::*;

fn main() {
    println_context!("context");
    println_context_value!("context_value");
    println_error!("error");
    println_header!("header");
    println_invalid!("invalid");
    println_literal!("literal");
    println_placeholder!("placeholder");
    println_usage!("usage");
    println_valid!("valid");
}
