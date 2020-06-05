use {anyhow::Error, icu_locale::Locale, pest::Parser, pest_derive::Parser, std::str::FromStr};

/// A single test case for
#[derive(Debug, Eq, PartialEq)]
pub struct LocaleMatcherTestCase {
    pub supported: Vec<Locale>,
    pub desired: Vec<Locale>,
    pub expected: Vec<Locale>,
}

#[derive(Parser)]
#[grammar = "../tests/helpers/locale_matcher_test_case.pest"]
pub struct LocaleMatcherTestCaseParser {}

impl FromStr for LocaleMatcherTestCase {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parsed = LocaleMatcherTestCaseParser::parse(Rule::test_case, s)?;
        let mut locale_lists: [Vec<Locale>; 3] = [vec![], vec![], vec![]];

        for (index, mut token_pair) in parsed.next().unwrap().into_inner().enumerate() {
            match token_pair.as_rule() {
                Rule::locale_list => {
                    let locale_list = &mut locale_lists[index];
                    let locale_list_pair = token_pair.into_inner();
                    for mut locale_pair in locale_list_pair {
                        match locale_pair.as_rule() {
                            Rule::locale => {
                                let locale_str = locale_pair.as_str();
                                let locale = locale_str.parse()?;
                                locale_list.push(locale);
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        let [supported, desired, expected] = locale_lists;
        Ok(LocaleMatcherTestCase {
            supported,
            desired,
            expected,
        })
    }
}

#[cfg(test)]
mod tests {
    use {super::*, std::str::FromStr};

    #[test]
    fn test_locale_matcher_test_case_from_str() -> Result<(), Error> {
        let actual: LocaleMatcherTestCase = "ab-CD de-EF; gh-HI; jk-LM # comment".parse()?;
        let expected = LocaleMatcherTestCase {
            supported: vec!["ab-CD".parse()?, "de-EF".parse()?],
            desired: vec!["gh-HI".parse()?],
            expected: vec!["jk-LM".parse()?],
        };
        assert_eq!(actual, expected);
        Ok(())
    }
}
