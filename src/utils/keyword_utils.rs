use regex::Regex;

pub fn parse_keywords(keyword_str: &str) -> Vec<String> {
    let regex = Regex::new(r#"(?m)[\\""].+?[\\""]|[^ ]+"#).unwrap();
    let result = regex.captures_iter(keyword_str);

    result
        .into_iter()
        .map(|cap| cap[0].to_string())
        .collect::<Vec<String>>()
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_whitespace_seperated_keyswords() {
        let keywords = super::parse_keywords("keyword1 keyword2 keyword3");
        assert_eq!(keywords.len(), 3);
    }

    #[test]
    fn parse_consider_quote_signed_keyswords() {
        let keywords = super::parse_keywords("keyword \"keyword keyword1\" keyword2");
        assert_eq!(keywords.len(), 3);
    }
}
