use {
    anyhow::Result, icu_locale::Locale, icu_locale_matcher::LocalePriorityList,
    pretty_assertions::assert_eq,
};

#[test]
fn test_weight_clamping() -> Result<()> {
    fn assert_expected_weight(original: f64, expected: Option<f64>) -> Result<()> {
        let locale: Locale = "und".parse()?;
        let mut builder = LocalePriorityList::builder();
        builder.add_with_weight(locale.clone(), original);
        let list = builder.build_with_weights();
        assert_eq!(list.get_weight(&locale), expected);
        Ok(())
    }

    assert_expected_weight(std::f64::NAN, None)?;
    assert_expected_weight(std::f64::NEG_INFINITY, None)?;
    assert_expected_weight(-1.0, None)?;
    assert_expected_weight(0.0, None)?;
    assert_expected_weight(0.5, Some(0.5))?;
    assert_expected_weight(1.0, Some(1.0))?;
    assert_expected_weight(5.0, Some(1.0))?;
    assert_expected_weight(std::f64::INFINITY, Some(1.0))?;

    Ok(())
}

#[test]
fn test_build_with_weights() -> Result<()> {
    type Entry = (Locale, f64);

    let entries: Vec<Entry> = vec![
        ("af".parse()?, 0.2),
        ("bo".parse()?, 0.4),
        ("cz".parse()?, 0.6),
        ("de".parse()?, 0.8),
        ("en".parse()?, 0.4),
    ];

    let mut builder = LocalePriorityList::builder();
    for entry in &entries {
        builder.add_with_weight(entry.0.clone(), entry.1);
    }
    let actual: Vec<Entry> = builder.build_with_weights().into_iter().collect();

    let expected: Vec<Entry> = vec![
        ("de".parse()?, 0.8),
        ("cz".parse()?, 0.6),
        ("bo".parse()?, 0.4),
        ("en".parse()?, 0.4),
        ("af".parse()?, 0.2),
    ];

    assert_eq!(actual, expected);

    Ok(())
}

#[test]
fn test_build_without_weights() -> Result<()> {
    type Entry = (Locale, f64);

    let entries: Vec<Entry> = vec![
        ("af".parse()?, 0.2),
        ("bo".parse()?, 0.4),
        ("cz".parse()?, 0.6),
        ("de".parse()?, 0.8),
        ("en".parse()?, 0.4),
    ];

    let mut builder = LocalePriorityList::builder();
    for entry in &entries {
        builder.add_with_weight(entry.0.clone(), entry.1);
    }
    let actual: Vec<Entry> = builder.build_without_weights().into_iter().collect();

    let expected: Vec<Entry> = vec![
        ("de".parse()?, 1.0),
        ("cz".parse()?, 1.0),
        ("bo".parse()?, 1.0),
        ("en".parse()?, 1.0),
        ("af".parse()?, 1.0),
    ];

    assert_eq!(actual, expected);

    Ok(())
}
