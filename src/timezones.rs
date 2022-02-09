use auto_correct_n_suggest::Dictionary;
use chrono_tz::Tz;

use crate::RizzyError;

pub fn parse_timezone(time_zone_str: &String) -> Result<Tz, RizzyError> {
    time_zone_str.parse().map_err(|_e| {
        let suggestions = guess_timezone(time_zone_str.to_string());
        RizzyError::InvalidTimezone(time_zone_str.to_string(), suggestions)
    })
}

pub fn guess_timezone(time_zone: String) -> Vec<String> {
    let mut dictionary = Dictionary::new();
    for tz in build_timezone_list() {
        dictionary.insert(tz);
    }

    let suggestions = dictionary.auto_suggest_alternative_words(time_zone);
    suggestions.unwrap_or_default()
}

fn build_timezone_list() -> Vec<String> {
    chrono_tz::TIMEZONES.keys().map(|x| x.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_spelling() {
        let guess = "America/Chicago";
        assert_eq!(
            parse_timezone(&guess.to_string()),
            chrono_tz::America::Chicago
        );
    }

    #[test]
    fn test_incorrect_spelling() {
        let possibilities = guess_timezone("America/Chirago".to_string());
        assert!(possibilities.contains(&"America/Chicago".to_string()));
    }
}
