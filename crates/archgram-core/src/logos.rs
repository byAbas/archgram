//! Technology logos (PRD, section 7; ARCHITECTURE.md, Render). The core
//! carries none: it draws the logos it is given, so the binary that ships
//! it decides which set it needs. The native CLI passes Simple Icons from
//! `archgram-icons`; a web build can pass only the few a diagram names.

use std::collections::BTreeMap;

/// A set of logos, each a single SVG path on a 24 by 24 grid, found by its
/// Simple Icons slug.
pub trait Logos {
    /// The path of the logo `slug` names, if the set has it.
    fn path(&self, slug: &str) -> Option<&str>;
    /// Every slug in the set, for suggesting one when a name is wrong.
    fn slugs(&self) -> Vec<&str>;

    /// The technology's name, for a suggestion and for a card's name that
    /// only repeats it.
    fn title(&self, _slug: &str) -> Option<&str> {
        None
    }

    /// The brand's colour, six hex digits, shown while a flow's signal is at
    /// the card (DESIGN.md, Components: Technology logo).
    fn colour(&self, _slug: &str) -> Option<&str> {
        None
    }
}

/// No logos at all: `tech` is then neither checked nor drawn.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoLogos;

impl Logos for NoLogos {
    fn path(&self, _: &str) -> Option<&str> {
        None
    }

    fn slugs(&self) -> Vec<&str> {
        Vec::new()
    }
}

impl Logos for BTreeMap<String, String> {
    fn path(&self, slug: &str) -> Option<&str> {
        self.get(slug).map(String::as_str)
    }

    fn slugs(&self) -> Vec<&str> {
        self.keys().map(String::as_str).collect()
    }
}

/// What a technology's name stands for in a set of logos, as `archgram
/// logos` prints it, so whoever writes a spec takes `tech` from the set the
/// drawing uses rather than guessing a slug.
#[derive(Debug, PartialEq, Eq)]
pub enum Found<'a> {
    /// One logo: its slug and its title.
    Logo(&'a str, &'a str),
    /// No one logo, but these, each slug with its title, start with the
    /// name or are near it.
    Candidates(Vec<(&'a str, &'a str)>),
    /// Nothing near.
    None,
}

/// A name as Simple Icons builds a slug from a title: lowercase, `.` as
/// `dot`, `+` as `plus`, `#` as `sharp`, `&` as `and`, everything else
/// that is not a letter or a digit left out.
fn as_slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars().flat_map(char::to_lowercase) {
        match c {
            '.' => out.push_str("dot"),
            '+' => out.push_str("plus"),
            '#' => out.push_str("sharp"),
            '&' => out.push_str("and"),
            c if c.is_alphanumeric() => out.push(c),
            _ => {}
        }
    }
    out
}

/// The logo `name` stands for: a product's name (`Next.js`, `Traefik`), a
/// package's (`sentry-sdk`, `@prisma/client`) or an image's (`adminer`).
/// The whole name first, as a slug or a title; then each word of it, the
/// longest first; then the titles a word starts. Up to five candidates.
#[must_use]
pub fn find<'a>(name: &str, logos: &'a dyn Logos) -> Found<'a> {
    let slugs = logos.slugs();
    let title = |slug: &'a str| logos.title(slug).unwrap_or(slug);
    let exact = |key: &str| {
        slugs
            .iter()
            .copied()
            .find(|s| *s == key || as_slug(title(s)) == key)
    };
    let whole = as_slug(name);
    if whole.is_empty() {
        return Found::None;
    }
    if let Some(s) = exact(&whole) {
        return Found::Logo(s, title(s));
    }
    let mut words: Vec<String> = name
        .split(|c: char| !c.is_alphanumeric() && !matches!(c, '.' | '+' | '#'))
        .map(as_slug)
        .filter(|w| w.chars().count() >= 3)
        .collect();
    words.sort_by_key(|w| std::cmp::Reverse(w.len()));
    for w in &words {
        if let Some(s) = exact(w) {
            return Found::Logo(s, title(s));
        }
    }
    let mut near: Vec<(&str, &str)> = slugs
        .iter()
        .copied()
        .filter(|s| {
            let first = title(s).split_whitespace().next().map(as_slug);
            first.as_deref() == Some(whole.as_str()) || s.starts_with(whole.as_str())
        })
        .map(|s| (s, title(s)))
        .collect();
    if near.is_empty() {
        near = crate::validate::nearest_few(&whole, slugs.iter().copied(), 5)
            .into_iter()
            .map(|s| (s, title(s)))
            .collect();
    }
    near.sort_unstable();
    near.truncate(5);
    match near.len() {
        0 => Found::None,
        1 => Found::Logo(near[0].0, near[0].1),
        _ => Found::Candidates(near),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Set(Vec<(&'static str, &'static str)>);

    impl Logos for Set {
        fn path(&self, slug: &str) -> Option<&str> {
            self.0.iter().find(|(s, _)| *s == slug).map(|_| "M0 0")
        }
        fn slugs(&self) -> Vec<&str> {
            self.0.iter().map(|(s, _)| *s).collect()
        }
        fn title(&self, slug: &str) -> Option<&str> {
            self.0.iter().find(|(s, _)| *s == slug).map(|(_, t)| *t)
        }
    }

    fn set() -> Set {
        Set(vec![
            ("sentry", "Sentry"),
            ("nextdotjs", "Next.js"),
            ("traefikproxy", "Traefik Proxy"),
            ("prisma", "Prisma"),
            ("googlecloud", "Google Cloud"),
            ("googlegemini", "Google Gemini"),
            ("postgresql", "PostgreSQL"),
        ])
    }

    #[test]
    fn a_product_or_a_package_finds_its_logo() {
        let s = set();
        assert_eq!(find("Next.js", &s), Found::Logo("nextdotjs", "Next.js"));
        assert_eq!(find("sentry-sdk", &s), Found::Logo("sentry", "Sentry"));
        assert_eq!(find("@prisma/client", &s), Found::Logo("prisma", "Prisma"));
        assert_eq!(
            find("Traefik", &s),
            Found::Logo("traefikproxy", "Traefik Proxy")
        );
        assert_eq!(
            find("PostgreSQL", &s),
            Found::Logo("postgresql", "PostgreSQL")
        );
    }

    #[test]
    fn a_name_of_several_gives_them_and_an_unknown_one_nothing() {
        let s = set();
        assert_eq!(
            find("google", &s),
            Found::Candidates(vec![
                ("googlecloud", "Google Cloud"),
                ("googlegemini", "Google Gemini")
            ])
        );
        assert_eq!(find("mailpit", &s), Found::None);
        assert_eq!(find("--", &s), Found::None);
    }
}
