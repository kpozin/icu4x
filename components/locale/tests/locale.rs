mod fixtures;
mod helpers;

use std::convert::TryInto;

use icu_locale::{Locale, ParserError};

type Result = std::result::Result<Locale, ParserError>;

fn test_langid_fixtures(tests: Vec<fixtures::LocaleTest>) {
    for test in tests {
        match test.output {
            fixtures::LocaleInfo::String(s) => {
                let input: Locale = test.input.try_into().expect("Parsing failed.");
                assert_eq!(input.to_string(), s);
            }
            fixtures::LocaleInfo::Error(err) => {
                let err: ParserError = err.into();
                let input: Result = test.input.try_into();
                assert_eq!(input, Err(err));
            }
            fixtures::LocaleInfo::Identifier(ident) => {
                let input: Locale = test.input.try_into().expect("Parsing failed.");
                let output: Locale = ident.try_into().expect("Parsing failed.");
                assert_eq!(input, output);
            }
            fixtures::LocaleInfo::Object(o) => {
                let input: Locale = test.input.try_into().expect("Parsing failed.");
                let output: Locale = o.try_into().expect("Parsing failed.");
                assert_eq!(input, output);
            }
        }
    }
}

#[test]
fn test_locale_parsing() {
    let path = "./tests/fixtures/locale.json";
    let data = helpers::read_fixture(path).expect("Failed to read a fixture");

    test_langid_fixtures(data);
}
