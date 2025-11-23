#![feature(lazy_cell)]

use std::{cmp::Ordering, collections::BTreeMap, io::stdin, str::FromStr, sync::LazyLock};

const PATH: &str = "./src/profanity.csv";

pub fn main() {
    let file = std::fs::read_to_string(PATH).unwrap();

    let mut entries = BTreeMap::<Phrase, Severity>::new();

    let header = file.lines().next().unwrap().to_owned();

    for line in file.lines().skip(1) {
        let (phrase, severity) = parse_line(line);

        let old = entries.insert(phrase, severity);
        assert!(old.is_none());
    }

    export(&header, &entries);

    let mut line = String::new();
    while let Ok(_) = stdin().read_line(&mut line) {
        let (phrase, severity) = parse_line(&line[0..line.len() - 1]);

        println!("adding {} with {severity:?}", phrase.0);

        let old = entries.insert(phrase, severity);
        assert!(old.is_none());

        export(&header, &entries);

        line.clear();
    }
}

type Severity = [u8; 5];

fn parse_line(line: &str) -> (Phrase, Severity) {
    let (phrase, severity) = line.split_once(',').expect(line);
    (
        Phrase(phrase.to_owned()),
        severity
            .split(',')
            .map(|n| u8::from_str(n).expect(line))
            .collect::<Vec<_>>()
            .try_into()
            .expect(line),
    )
}

fn export(header: &str, entries: &BTreeMap<Phrase, Severity>) {
    let mut output = format!("{header}\n");
    for (phrase, severity) in entries {
        use std::fmt::Write;
        writeln!(
            output,
            "{},{}",
            phrase.0,
            severity
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
    }

    std::fs::write(PATH, output).unwrap();
}

fn is_emoji(input: &str) -> bool {
    static EMOJI_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
        const EMOJI_REGEX: &str = r"^\p{Extended_Pictographic}(\p{EMod}|\x{FE0F}\x{20E3}?|[\x{E0020}-\x{E007E}]+\x{E007F})?(\x{200D}(\p{RI}\p{RI}|\p{Extended_Pictographic}(\p{EMod}|\x{FE0F}\x{20E3}?|[\x{E0020}-\x{E007E}]+\x{E007F})?))*$";
        regex::Regex::new(EMOJI_REGEX).unwrap()
    });

    EMOJI_REGEX.is_match(&input)
}

fn is_cyrillic(c: char) -> bool {
    matches!(c, '\u{0400}'..='\u{04FF}' // Cyrillic
            | '\u{0500}'..='\u{052F}' // Cyrillic Supplementary
            | '\u{2DE0}'..='\u{2DFF}' // Cyrillic Extended-A
            | '\u{A640}'..='\u{A69F}' // Cyrillic Extended-B
            | '\u{FE2E}'..='\u{FE2F}') // Combining Half Marks (some used with Cyrillic)
}

pub fn is_cjk(c: char) -> bool {
    let cp: u32 = c.into();
    (cp >= 0x4E00 && cp <= 0x9FFF)
        || (cp >= 0x3400 && cp <= 0x4DBF)
        || (cp >= 0x20000 && cp <= 0x2A6DF)
        || (cp >= 0x2A700 && cp <= 0x2B73F)
        || (cp >= 0x2B740 && cp <= 0x2B81F)
        || (cp >= 0x2B820 && cp <= 0x2CEAF)
        || (cp >= 0xF900 && cp <= 0xFAFF)
        || (cp >= 0x2F800 && cp <= 0x2FA1F)
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Phrase(String);

#[derive(Eq, PartialEq, PartialOrd, Ord)]
enum Class {
    AnyEmoji,
    Other,
    AllAscii,
    AnyCyrillic,
    AnyCjk,
}

impl Phrase {
    fn trim(&self) -> String {
        self.0.trim_start().to_ascii_lowercase()
    }

    fn class(&self) -> Class {
        let s = self.trim();
        if s.chars().any(|c| is_emoji(&format!("{c}"))) {
            Class::AnyEmoji
        } else if s.bytes().take(3).collect::<Vec<_>>().is_ascii() {
            Class::AllAscii
        } else if s.chars().any(is_cyrillic) {
            Class::AnyCyrillic
        } else if s.chars().any(is_cjk) {
            Class::AnyCjk
        } else {
            Class::Other
        }
    }
}

impl Ord for Phrase {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.class(), self.trim()).cmp(&(other.class(), other.trim()))
    }
}

impl PartialOrd for Phrase {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
