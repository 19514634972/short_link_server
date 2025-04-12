use regex::Regex;

pub fn is_five_letters(s: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z]{5}$").unwrap();
    re.is_match(s)
}


