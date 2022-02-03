use ::lazy_static::lazy_static;
use ::regex::Match;
use ::regex::Regex;
use crate::dvb::data::Tag;

lazy_static! {
    static ref TAG_DIGITS_RE: Regex = Regex::new(r"[0-9]+").unwrap();
}

fn tag_re_str(tag_str: &str) -> String {
    let tag_escaped_for_re = &tag_str.replace('-', r"\-").replace('.', r"\.");
    let tag_digits_replaced = TAG_DIGITS_RE.replace_all(tag_escaped_for_re, "([0-9]+)");
    tag_digits_replaced.into_owned()
}

pub(crate) fn tag_to_re(tag_str: &str) -> Result<Regex, String> {
    let tag_digits_replaced = tag_re_str(tag_str);
    let tag_full_match_re = format!("^{}$", tag_digits_replaced);
    let regex = Regex::new(tag_full_match_re.as_ref()).map_err(|err| {
        format!(
            "tag could not be turned into regex pattern; tag: {}, err: {}",
            tag_str, err
        )
    })?;
    Ok(regex)
}

pub(crate) fn image_tag_to_re(image: &str, tag: &str, suffix: &str) -> Result<Regex, String> {
    let tag_digits_replaced = tag_re_str(tag);
    let pattern_str = format!("^FROM\\s+{}:{}\\s+{}$", image, tag_digits_replaced, suffix);
    let pattern = Regex::new(&pattern_str).map_err(|err| {
        format!("image and tag could not be turned into regex pattern; {}, err: {}", pattern_str, err) })?;
    Ok(pattern)
}

pub fn parse_tag(tag_pattern: &Regex, tag: impl Into<String>) -> Result<Tag, String> {
    let tag = tag.into();
    let parts = tag_pattern.captures(&tag).ok_or_else(|| {
        format!(
            "could not extract digits from tag; tag: {}, pattern: {}, failed to capture",
            &tag,
            tag_pattern.as_str()
        )
    })?;
    let nrs = (
        match_to_nr(parts.get(1)),
        match_to_nr(parts.get(2)),
        match_to_nr(parts.get(3)),
        match_to_nr(parts.get(4)),
    );
    Ok(Tag::new(tag, nrs))
}

fn match_to_nr(mtch: Option<Match>) -> u32 {
    mtch.map(|mtch| mtch.as_str().parse::<u32>().unwrap())
        .unwrap_or_else(|| 0)
}
