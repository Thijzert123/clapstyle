use clap::builder::{Styles, styling::Style};
use duplicate::duplicate_item;
use parking_lot::RwLock;
use pastey::paste;
use std::fmt::Write;
use std::{cell::RefCell, fmt};

#[doc(hidden)]
pub use anstream;

pub static CLAP_STYLES: RwLock<Styles> = RwLock::new(Styles::styled());
thread_local! {
    static STYLE_STACK: RefCell<Vec<Style>> = RefCell::new(Vec::new());
}

fn push_style(style: Style) -> Style {
    STYLE_STACK.with(|s| {
        let mut s = s.borrow_mut();
        s.push(style)
    });
    style
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
    write!(out, "{}", push_style(style.clone())).unwrap();
    out.write_fmt(args).unwrap();
    write!(out, "{}", pop_style_and_restore()).unwrap();
    out
}

// TODO document everything
// TODO change crate name

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
    #[doc = "`" print_type "` with Clap's `" style_type "` style"]
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

#[duplicate_item(
    print_type;
    [print];
    [println];
    [eprint];
    [eprintln];
    [panic];
)]
paste! {
    #[doc = "`" print_type "` without a style"]
    #[doc = ""]
    #[doc = "Drop-in replacement for `" print_type "!` macro."]
    #[doc = "The text gets passed through [`anstream`]."]
    #[doc = "This means that possible manually applied styles get removed if, for example, the output is piped."]
    #[macro_export]
    macro_rules! print_type {
        ($($arg:tt)*) => {
            $crate::anstream::print_type!($($arg)*);
        };
    }
}

pub trait ClapStylize {
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
        fn [<style_ style_type>](&self) -> ClapStyledValue<'_, Self>
        where
            Self: Sized
        {
            self.clap_styled(CLAP_STYLES.read().[<get_ style_type>]().clone())
        }
    }
}

impl<T> ClapStylize for T {}

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
        write!(f, "{}", push_style(self.style))?;
        fmt::trait_name::fmt(self.value, f)?;
        write!(f, "{}", pop_style_and_restore())
    }
}

#[cfg(feature = "anyhow")]
pub type Result<T, E = Error> = core::result::Result<T, E>;

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
