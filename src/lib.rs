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

// TODO also implement anyhow result styling
// TODO handle panic
// TODO document everything
// TODO convenience methods for for example errors

#[duplicate_item(
    print_type;
    [print];
    [println];
    [eprint];
    [eprintln];
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
