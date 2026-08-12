#![cfg_attr(docsrs, feature(doc_cfg))]

//! Simple utility library to match your CLI's output to Clap's output.
//! This means that your, headers, errors, literals, etc. all have the same style as Clap.
//!
//! ## Macros
//! You can use one of the macros to output something in a style you select.
//! The macros are a drop-in replacement for whatever you otherwise would have used
//! ([`println_header`] accepts the same arguments as [`std::println`]).
//! The [`print`], [`println`], [`panic`], etc. macros are re-exported from `anstream`.
//! They don't apply a style to the output.
//!
//! This example prints a line to `stderr` with Clap's `error` style:
//! ```
//! use clapstyle::eprintln_error;
//!
//! eprintln_error!("Something went wrong!");
//! ```
//! If the default style is used, the full text will be bold and red.
//!
//! ## Style methods
//! It is possible to style all types using the [`ClapStylize`] trait. That means that you
//! can use [`std::println`] with one of Clap's styles:
//! ```
//! use clapstyle::ClapStylize;
//!
//! println!("This is a {} and this is an {}", "header".style_header(), "error".style_error());
//! ```
//! However, because you apply style codes to the output, you should route it via this crates
//! [`println`]. This is so that style codes get removed if the output is, for example, piped to a file.
//! ```
//! use clapstyle::println;
//! use clapstyle::ClapStylize;
//!
//! println!("This is a {} and this is an {}", "header".style_header(), "error".style_error());
//! ```
//!
//! ## Nested styles
//! You can combine the style macros with the style methods. This allows you to have multiple
//! styles in one line:
//! ```
//! use clapstyle::println_error;
//! use clapstyle::ClapStylize;
//!
//! println_error!("Error with a '{}'. This is still an error.", "literal".style_literal());
//! ```
//! The output is styled like this:
//! ```text
//! Error with a 'literal'. This is still an error.
//! \------------/\-----/\------------------------/
//!     |           |             |
//!  error style    |         error style
//!              literal style
//! ```
//! The `error` style gets reset after. This works because all styles get pushed on a style stack.
//! Frist, the `error` style gets pushed and applied.
//! Then, when formatting the `literal` style, a new style first gets pushed (`literal`) and applied.
//! Then, it gets popped and the previous last style gets applied (in this case `error`).
//! When the last item gets popped, [`anstyle::Reset`] gets applied.
//!
//! ## `anyhow` compatibility
//! Using the `anyhow` feature flag, you can print `anyhow` errors in Clap style.
//! Please see [`Result`] for how to use and [`Error`] for the errors are displayed.
//!
//! ## Change Clap's style
//! If you want to change the style of all Clap-styled output, you can modify the [`CLAP_STYLES`]
//! variable. This doesn't change Clap's output, however.
//!
//! ## Examples
//! Please take a look at the examples available in the `examples` directory in the repository.
//! You can run them with Cargo:
//! ```bash
//! cargo run --example demo
//! # or
//! cargo run --example anyhow --features=anyhow
//! ```

use clap::builder::{Styles, styling::Style};
use duplicate::duplicate_item;
use parking_lot::RwLock;
use pastey::paste;
use std::fmt::Write;
use std::{cell::RefCell, fmt};

// For doc comments
#[allow(unused_imports)]
use std::fmt::Display;

// Only for the macros so that users don't have to import anstream manually.
#[doc(hidden)]
pub use anstream;

#[duplicate_item(
    print_type;
    [print];
    [println];
    [eprint];
    [eprintln];
    [panic];
)]
pub use anstream::print_type;

/// Styles that Clap uses.
///
/// Defaults to Clap's defaults.
/// You can change these styles to whatever [`Styles`] you want.
/// This does not change Clap's style usage.
pub static CLAP_STYLES: RwLock<Styles> = RwLock::new(Styles::styled());

thread_local! {
    static STYLE_STACK: RefCell<Vec<Style>> = RefCell::new(Vec::new());
}

fn push_style_and_apply(style: Style) -> String {
    STYLE_STACK.with(|s| s.borrow_mut().push(style));
    format!("{}{}", anstyle::Reset, style)
}

fn pop_style_and_restore() -> String {
    let restore_style = STYLE_STACK.with(|s| {
        let mut s = s.borrow_mut();
        s.pop();
        s.last().copied()
    });
    match restore_style {
        Some(restore_style) => format!("{}", restore_style),
        None => format!("{}", anstyle::Reset),
    }
}

#[doc(hidden)]
pub fn wrap_style(style: &Style, args: fmt::Arguments<'_>) -> String {
    let mut out = String::new();
    write!(out, "{}", push_style_and_apply(style.clone())).unwrap();
    out.write_fmt(args).unwrap();
    write!(out, "{}", pop_style_and_restore()).unwrap();
    out
}

#[duplicate_item(
    print_type;
    [print];
    [println];
    [eprint];
    [eprintln];
    [panic];
)]
#[duplicate_item(
    style_type;
    [context];
    [context_value];
    [error];
    [header];
    [valid];
    [invalid];
    [literal];
    [placeholder];
    [usage];
)]
paste! {
    #[doc = "`" print_type "` with Clap's `" style_type "` style."]
    #[doc = ""]
    #[doc = "Drop-in replacement for `" print_type "!` macro."]
    #[doc = "The style Clap uses for `" style_type "` gets applied to everything that gets passed."]
    #[doc = "Afterward, the style resets so all other text do not have this style."]
    #[doc = ""]
    #[doc = "The text gets passed through [`anstream`]."]
    #[doc = "This means that the applied style gets removed if, for example, the output is piped."]
    #[macro_export]
    macro_rules! [<print_type _ style_type>] {
        () => {
            $crate::anstream::print_type!()
        };
        ($($arg:tt)*) => {{
            let clap_styles = $crate::CLAP_STYLES.read();
            let style = clap_styles.[<get_ style_type>]();
            let out = $crate::wrap_style(style, ::std::format_args!($($arg)*));
            $crate::anstream::print_type!("{}", out);
        }};
    }
}

/// Trait to stylize items with Clap's default styles.
///
/// This trait is implemented for all types.
pub trait ClapStylize {
    /// Convert `Self` into a [`ClapStyledValue`] with a given [`Style`].
    fn clap_styled(&self, style: Style) -> ClapStyledValue<'_, Self>
    where
        Self: Sized,
    {
        ClapStyledValue { value: self, style }
    }

    #[duplicate::duplicate_item(
        style_type;
        [context];
        [context_value];
        [error];
        [header];
        [invalid];
        [literal];
        [placeholder];
        [usage];
        [valid];
    )]
    paste! {
        #[doc = "Style `Self` in Clap's `" style_type "` style."]
        fn [<style_ style_type>](&self) -> ClapStyledValue<'_, Self>
        where
            Self: Sized
        {
            self.clap_styled(CLAP_STYLES.read().[<get_ style_type>]().clone())
        }
    }
}

impl<T> ClapStylize for T {}

/// Value with a style.
///
/// This structs implements any std formatter ([`Display`], [`Debug`], etc.).
/// When formatting using one of these, the style is applied to the value.
/// It automatically supports [nested styles](index.html#nested-styles).
pub struct ClapStyledValue<'a, T> {
    value: &'a T,
    style: Style,
}

#[duplicate::duplicate_item(
    trait_name;
    [Display];
    [Debug];
    [UpperHex];
    [LowerHex];
    [Binary];
    [UpperExp];
    [LowerExp];
    [Octal];
    [Pointer];
)]
impl<'a, T: fmt::trait_name> fmt::trait_name for ClapStyledValue<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Make sure things like aligning keep working
        write!(f, "{}", push_style_and_apply(self.style))?;
        fmt::trait_name::fmt(self.value, f)?;
        write!(f, "{}", pop_style_and_restore())
    }
}

/// [`Result`] with `clapstyle`s own error type.
///
/// This type is meant to be a replacement for [`anyhow::Result`] if you want Clap's styling.
/// You can convert any error that is compatible with `anyhow` to this type with the `?` operator:
///
/// ```
/// fn returns_clapstyle_result() -> clapstyle::Result<()> {
///     operation_that_fails()?;
///     println!("Operation complete");
///     Ok(())
/// }
///
/// fn operation_that_fails() -> anyhow::Result<()> {
///     std::fs::read_to_string("doesnt_exist")?;
///     println!("Read file complete");
///     Ok(())
/// }
/// ```
///
/// Just like [`anyhow::Result`], setting it as the return value of `main()` makes sure any errors get
/// pretty prined using the [`Debug`] trait.
#[cfg(feature = "anyhow")]
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// A wrapper around [`anyhow::Error`].
///
/// You can convert any type that is also able to convert into [`anyhow::Error`] to this type.
///
/// Just like [`anyhow::Error`], [`Display`] and [`Debug`] are implemented. These are implemented
/// so that they mimic `anyhow`s style as closely as possbile, but with Clap's style.
/// Below is what they look like. Keep in mind `error:` is bold red by default (see [`CLAP_STYLES`]),
/// and `Causes:` or `Stack backtrace:` is bold underlined.
///
/// ### Default [`Display`] (`{}`)
/// ```text
/// error: couldn't process file
/// ```
///
/// ### Alternative [`Display`] (`{:#}`)
/// ```text
/// error: couldn't process file: couldn't open file
/// ```
///
/// ### Default [`Debug`] (`{:?}`)
/// ```text
/// error: couldn't process file
///
/// Caused by:
///     0: couldn't open file
///     1: no such file or directory (os error 2)
/// ```
/// If a stack backtrace is available (some omitted for readability):
/// ```text
/// error: couldn't process file
///
/// Caused by:
///     0: couldn't open file
///     1: no such file or directory (os error 2)
///
/// Stack backtrace:
///     0: <std::io::error::Error as anyhow::context::ext::StdError>::ext_context::<&str>
///               at /home/thijs/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.104/src/backtrace.rs:10:14
///    21: __libc_start_main
///    22: _start
/// ```
///
///
/// ### Alternative [`Debug`] (`{:#?}`)
/// (default debug display, nothing is styled)
/// ```text
/// Error {
///     context: "Couldn\'t process file",
///     source: Error {
///         context: "Couldn\'t open file",
///         source: Os {
///             code: 2,
///             kind: NotFound,
///             message: "No such file or directory",
///         },
///     },
/// }
/// ```
#[cfg(feature = "anyhow")]
pub struct Error(anyhow::Error);

#[cfg(feature = "anyhow")]
mod error_impl {
    use std::backtrace::BacktraceStatus;
    use std::fmt::Debug;
    use std::fmt::Display;

    use crate::ClapStylize;
    use crate::Error;

    fn first_char_lowercase(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            Some(first_char) => first_char.to_lowercase().chain(chars).collect(),
            None => String::new(),
        }
    }

    impl<E: Into<anyhow::Error>> From<E> for Error {
        fn from(anyhow_error: E) -> Self {
            Error(anyhow_error.into())
        }
    }

    impl Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut chain = self.0.chain();
            let error = first_char_lowercase(&format!("{}", chain.next().unwrap()));
            write!(f, "{} {}", "error:".style_error(), error)?;

            if f.alternate()
                && let Some(cause) = chain.next()
            {
                write!(f, ": {}", first_char_lowercase(&format!("{}", cause)))?;
            }

            Ok(())
        }
    }

    impl Debug for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if f.alternate() {
                return write!(f, "{:#?}", self.0);
            }

            let mut chain = self.0.chain();
            let error = first_char_lowercase(&format!("{}", chain.next().unwrap()));
            writeln!(f, "{} {}", "error:".style_error(), error)?;
            writeln!(f)?;
            write!(f, "{}", "Caused by:".style_header())?;

            if chain.len() == 1 {
                // No extra causes
                writeln!(f)?;
                write!(f, "    {}", chain.next().unwrap())?;
            } else if chain.len() > 1 {
                // Chain has numbered causes
                for (i, cause) in chain.enumerate() {
                    let cause = first_char_lowercase(&format!("{}", cause));
                    writeln!(f)?;
                    write!(f, "    {}: {}", i, cause)?;
                }
            }

            let backtrace = self.0.backtrace();
            if backtrace.status() == BacktraceStatus::Captured {
                writeln!(f)?;
                writeln!(f)?;
                writeln!(f, "{}", "Stack backtrace:".style_header())?;

                let mut backtrace = backtrace.to_string();
                backtrace.truncate(backtrace.trim_end().len());
                // Add 1 space to indentation to make it consistent with 'Caused by:' tree.
                let backtrace = backtrace
                    .lines()
                    .map(|line| format!(" {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                write!(f, "{}", backtrace)?;
            }

            Ok(())
        }
    }
}
