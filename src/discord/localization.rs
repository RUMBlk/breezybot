//Code in this file is taken from the Poise's Fluent translation example, some of it was edited to fit the codebase
use crate::{Data, Error, localization::{ Translations, format } };

/// Macro to retrieve a translation, optionally with arguments. Use like:
/// - `loc!(ctx, "identifier")` (no arguments)
/// - `loc!(ctx, "identifier", arg1: VALUE1, arg2: VALUE2)` (with arguments)
/// - 'loc!(ctx, "identifier", "attribute")' (with attribute)
/// - 'loc!(ctx, "identifier", "attribute", arg1: VALUE1, arg2: VALUE2)' (with attribute and arguments)
///
/// Does support retrieving message attributes
macro_rules! loc {
    ($ctx:expr, $id:expr $(, $argname:ident: $argvalue:expr )* $(,)?) => {
        crate::loc!($ctx.data().translations, $ctx.locale(), $id $(, $argname: $argvalue )*)
    };
    
    ($ctx:expr, $id:expr, $attr:expr $(, $argname:ident: $argvalue:expr )* $(,)?) => {
        crate::loc!($ctx.data().translations, $ctx.locale(), $id, $attr $(, $argname: $argvalue )*)
    };
}
pub(super) use loc;

/// Given a set of language files, fills in command strings and their localizations accordingly
pub fn apply_translation(
    translations: &Translations,
    commands: &mut [poise::Command<Data, Error>],
) {
    for command in &mut *commands {
        for mut subcommand in &mut command.subcommands {
            translate_ds_cmd(translations, format!("{}-{}", &command.name, subcommand.name).as_str(), &mut subcommand);
        }
        translate_ds_cmd(translations, &command.name.clone(), command);
    }
}


/// Given a set of language files, fills in command strings and their localizations accordingly
pub fn translate_ds_cmd(
    translations: &Translations,
    ftl_key: &str,
    command: &mut poise::Command<Data, Error>,
) {
    for (locale, bundle) in translations.other() {
        // Insert localized command name and description
        if let Some(description) = format(bundle, ftl_key, Some("description"), None) {
            command.description_localizations.insert(
                locale.clone(),
                description,
            );
        }

        for parameter in &mut command.parameters {

            if let Some(name) = format(bundle, ftl_key, Some(&parameter.name), None) {
                parameter.name_localizations.insert(
                    locale.clone(),
                    name,
                );
            }
            if let Some(name) = format(
                bundle,
                ftl_key,
                Some(&format!("{}-description", parameter.name)),
                None,
            ) {
                parameter.description_localizations.insert(
                    locale.clone(),
                    name,
                );
            }

            // If this is a choice parameter, insert its localized variants
            for choice in &mut parameter.choices {
                if let Some(name) = format(bundle, &choice.name, None, None) {
                    choice.localizations.insert(
                        locale.clone(),
                        name,
                    );
                }
            }
        }
    }

    // At this point, all translation files have been applied. However, if a user uses a locale
    // we haven't explicitly inserted, there would be no translations at all -> blank texts. So,
    // we use the "main" translation file (en-US) as the non-localized strings.

    // Set fallback command name and description to en-US
    let bundle = translations.main();
    if let Some(x) = format(bundle, ftl_key, None, None) {
        command.name = x;
    }

    command.description =
        format(bundle, ftl_key, Some("description"), None);

    /*if let Some(txt) = &command.description {
        eprintln!("e: {}", txt);
    }*/

    for parameter in &mut command.parameters {
        // Set fallback parameter name and description to en-US
        if let Some(name) = format(bundle, ftl_key, Some(&parameter.name), None) {
            parameter.name = name;
        }

        parameter.description = format(
                bundle,
                ftl_key,
                Some(&format!("{}-description", parameter.name)),
                None,
        );

        // If this is a choice parameter, set the choice names to en-US
        for choice in &mut parameter.choices {
            if let Some(name) = format(bundle, &choice.name, None, None) {
                choice.name = name;
            }
        }
    }
}
