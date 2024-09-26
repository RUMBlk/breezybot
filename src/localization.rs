//Code in this file is taken from the Poise's Fluent translation example, some of it was edited to fit the codebase
//! Wraps the fluent API and provides easy to use functions and macros for translation

use crate::Error;
macro_rules! loc {
    ( $translations:expr, $locale:expr, $id:expr $(, $argname:ident: $argvalue:expr )* $(,)? ) => {{
        #[allow(unused_mut)]
        let mut args = fluent::FluentArgs::new();
        $( args.set(stringify!($argname), $argvalue); )*

        $translations.get($locale, $id, None, Some(&args))
    }};
    ( $translations:expr, $locale:expr, $id:expr ) => {{
        $translations.get($locale, $id, None, None)
    }};
    ( $translations:expr, $locale:expr, $id:expr, $attr:expr ) => {{
        $translations.get($locale, $id, Some($attr), None)
    }};
    ( $translations:expr, $locale:expr, $id:expr, $attr:expr $(, $argname:ident: $argvalue:expr )* $(,)? ) => {{
        #[allow(unused_mut)]
        let mut args = fluent::FluentArgs::new();
        $( args.set(stringify!($argname), $argvalue); )*

        $translations.get($locale, $id, Some($attr), Some(&args))
    }};
}
pub(super) use loc;

type FluentBundle = fluent::bundle::FluentBundle<
    fluent::FluentResource,
    intl_memoizer::concurrent::IntlLangMemoizer,
>;

pub struct Translations {
    main: FluentBundle,
    other: std::collections::HashMap<String, FluentBundle>,
}

impl Translations {
    pub fn get(
        &self,
        locale: Option<&str>,
        id: &str,
        attr: Option<&str>,
        args: Option<&fluent::FluentArgs<'_>>,
    ) -> String {
        locale
            // Try to get the language-specific translation
            .and_then(|locale| format(self.other.get(locale)?, id, attr, args))
            // Otherwise, fall back on main translation
            .or_else(|| format(&self.main, id, attr, args))
            // If this message ID is not present in any translation files whatsoever
            .unwrap_or_else(|| {
                tracing::warn!("unknown fluent message identifier `{}`", id);
                id.to_string()
            })
    }

    pub fn main(&self) -> &FluentBundle {
        &self.main
    }

    pub fn other(&self) -> &std::collections::HashMap<String, FluentBundle> {
        &self.other
    }
}

/// Given a language file and message identifier, returns the translation
pub fn format(
    bundle: &FluentBundle,
    id: &str,
    attr: Option<&str>,
    args: Option<&fluent::FluentArgs<'_>>,
) -> Option<String> {
    let message = bundle.get_message(id)?;
    let pattern = match attr {
        Some(attribute) => message.get_attribute(attribute)?.value(),
        None => message.value()?,
    };
    let formatted = bundle.format_pattern(pattern, args, &mut vec![]);
    Some(formatted.into_owned())
}

/// Parses the `translations/` folder into a set of language files (FluentBundle)
pub fn read_ftl() -> Result<Translations, Error> {
    fn read_single_ftl(path: &std::path::Path) -> Result<(String, FluentBundle), Error> {
        // Extract locale from filename
        let locale = path.file_stem().ok_or("invalid .ftl filename")?;
        let locale = locale.to_str().ok_or("invalid filename UTF-8")?;

        // Load .ftl resource
        let file_contents = std::fs::read_to_string(path)?;
        let resource = fluent::FluentResource::try_new(file_contents)
            .map_err(|(_, e)| format!("failed to parse {:?}: {:?}", path, e))?;

        // Associate .ftl resource with locale and bundle it
        let mut bundle = FluentBundle::new_concurrent(vec![locale
            .parse()
            .map_err(|e| format!("invalid locale `{}`: {}", locale, e))?]);
        bundle
            .add_resource(resource)
            .map_err(|e| format!("failed to add resource to bundle: {:?}", e))?;

        Ok((locale.to_string(), bundle))
    }

    Ok(Translations {
        main: read_single_ftl("translations/en-US.ftl".as_ref())?.1,
        other: std::fs::read_dir("translations")?
            .map(|file| read_single_ftl(&file?.path()))
            .collect::<Result<_, _>>()?,
    })
}