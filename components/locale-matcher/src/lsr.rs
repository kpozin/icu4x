use {
    bitflags::bitflags,
    icu_locale::subtags::{Language, Region, Script},
};

bitflags! {
    struct LsrFlags: u8 {
        const EXPLICIT_REGION = 0b0000_0001;
        const EXPLICIT_SCRIPT = 0b0000_0010;
        const EXPLICIT_LANGUAGE = 0b0000_01000;
    }
}

pub struct LanguageScriptRegion {
    pub language: Language,
    pub script: Script,
    pub region: Region,
}
