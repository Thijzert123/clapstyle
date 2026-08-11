use clap::builder::Styles;
use duplicate::duplicate_item;
use parking_lot::RwLock;
use pastey::paste;

#[doc(hidden)]
pub use anstream;

pub static CLAP_STYLES: RwLock<Styles> = RwLock::new(Styles::styled());

// TODO also implement anyhow result styling
// TODO handle panic
// TODO document everything
// TODO convenience methods for for example errors
// TODO get style (re-export?)

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
           $crate::anstream::print_type!("{}{}{:#}", style, ::std::format_args!($($arg)*), style);
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
