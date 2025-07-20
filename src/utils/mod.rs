use std::collections::HashSet;

pub fn is_string_vec_unique<E>(vec: &[&str], error: E) -> Result<bool, E> {
    let mut seen = HashSet::new();
    if vec.iter().all(|&s| seen.insert(s)) {
        Ok(true)
    } else {
        Err(error)
    }
}

pub fn is_vec_len_gt_1<E>(vec: &[&str], error: E) -> Result<bool, E> {
    if vec.is_empty() || vec.len() == 1 {
        Err(error)
    } else {
        Ok(true)
    }
}

pub fn is_abbrev_match_to_string(abbrev: &str, name: &str) -> bool {
    // Check if at least one or more characters provided as an "abbreviation"
    // are present in a string i.e. it is a valid abbreviation.
    abbrev.chars().all(|c| name.chars().any(|nc| nc == c))
}

pub fn strs_to_strings(values: Vec<&str>) -> Vec<String> {
    values.iter().map(|x| x.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::is_abbrev_match_to_string;

    #[test]
    fn match_person_to_name() {
        assert_eq!(is_abbrev_match_to_string("Hn", "Hannah"), true);
        assert_eq!(is_abbrev_match_to_string("Hh", "Hannah"), true);
        assert_eq!(is_abbrev_match_to_string("Hb", "Hannah"), false);
    }
}
