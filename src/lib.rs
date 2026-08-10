use clap::builder::Styles;
use duplicate::duplicate_item;
use pastey::paste;
use std::sync::OnceLock;

#[doc(hidden)]
pub use anstream;

pub fn get_clap_styles() -> &'static Styles {
    static CLAP_STYLE: OnceLock<Styles> = OnceLock::new();
    CLAP_STYLE.get_or_init(|| Styles::styled())
}

// TODO also implement anyhow result styling
// TODO handle panic

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
           $crate::anstream::println!()
       };
       ($($arg:tt)*) => {{
           let style = $crate::get_clap_styles().[<get_ style_type>]();
           $crate::anstream::print_type!("{}{}{:#}", style, ::std::format_args!($($arg)*), style);
       }};
   }
}
