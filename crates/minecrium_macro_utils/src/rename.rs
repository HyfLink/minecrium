use proc_macro2::{Ident, Span};
use syn::{Error, LitStr, Result};

/// Renames an identifier by the specified style.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RenameStyle {
    /// keeps as it is.
    #[default]
    Default,
    /// makes all characters lowercase.
    Lowercase,
    /// makes all characters uppercase.
    Uppercase,
    /// lowercase, separates words by underscores.
    Snakecase,
}

impl RenameStyle {
    /// Parses the rename style from the string literal.
    pub fn new(style: &str, span: Span) -> Result<Self> {
        const MESSAGE: &str = "";

        match style {
            "default" => Ok(Self::Default),
            "lowercase" => Ok(Self::Lowercase),
            "uppercase" => Ok(Self::Uppercase),
            "snakecase" => Ok(Self::Snakecase),
            _ => Err(Error::new(span, MESSAGE)),
        }
    }

    /// Applies the style to the identifier.
    pub fn apply(&self, name: &Ident) -> LitStr {
        fn make_ascii_snakecase(s: &str) -> String {
            let mut snakecase = String::with_capacity(s.len());
            let mut is_last_uppercase = false;
            let mut is_leading_char = true;

            for ch in s.chars() {
                if !is_leading_char && !is_last_uppercase && ch.is_ascii_uppercase() {
                    snakecase.push('_');
                }

                is_leading_char = false;
                is_last_uppercase = ch.is_ascii_uppercase();
                snakecase.push(ch.to_ascii_lowercase());
            }

            snakecase
        }

        let mut value = name.to_string();
        match self {
            Self::Default => (),
            Self::Lowercase => value.make_ascii_lowercase(),
            Self::Uppercase => value.make_ascii_uppercase(),
            Self::Snakecase => value = make_ascii_snakecase(&value),
        }

        LitStr::new(&value, name.span())
    }
}

impl syn::parse::Parse for RenameStyle {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let style: LitStr = input.parse()?;
        Self::new(&style.value(), style.span())
    }
}
