//! Guest-typed text quoted in module emails.
//!
//! The record keeps the full text. The email quotes at most [`GUEST_TEXT_EMAIL_MAX_CHARS`]
//! chars of each guest-typed field, so its body stays under
//! `portaki_sdk::limits::EMAIL_BODY_MAX_CHARS` whatever was typed, and the CTA reads
//! « Voir plus » when something was cut.
//!
//! Official modules share no crate besides the SDK: lost-found, consumables, issue-report and
//! guest-reviews each carry this same file. Change them together.

use portaki_sdk::host::email::LocalizedEmailText;
use portaki_sdk::host::log;
use portaki_sdk::PortakiError;

/// Longest guest-typed field quoted in an email, in chars (not bytes).
pub const GUEST_TEXT_EMAIL_MAX_CHARS: usize = 500;

/// How far back from the cut a word boundary is looked for, in chars.
const WORD_BOUNDARY_WINDOW_CHARS: usize = 40;

/// `email_i18n` key of the CTA label used when a quote was cut.
const SEE_MORE_CTA_KEY: &str = "email.cta.seeMore";

/// A field as quoted in an email.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quoted {
    /// The quoted copy, ending in `…` when cut.
    pub text: String,
    /// Whether the original was longer than the quote.
    pub truncated: bool,
}

/// `text` quoted within [`GUEST_TEXT_EMAIL_MAX_CHARS`].
pub fn quote_guest_text(text: &str) -> Quoted {
    clip_chars(text, GUEST_TEXT_EMAIL_MAX_CHARS)
}

/// `text` cut to at most `max` chars, then `…` when cut.
///
/// The cut backs off to the last whitespace within the final [`WORD_BOUNDARY_WINDOW_CHARS`]
/// chars when there is one, so a word is not split; otherwise it falls exactly at `max`.
pub fn clip_chars(text: &str, max: usize) -> Quoted {
    let Some((cut, _)) = text.char_indices().nth(max) else {
        return Quoted {
            text: text.to_string(),
            truncated: false,
        };
    };
    let head = &text[..cut];
    let end = if text[cut..].starts_with(char::is_whitespace) {
        head.len()
    } else {
        let window_start = head
            .char_indices()
            .rev()
            .nth(WORD_BOUNDARY_WINDOW_CHARS.saturating_sub(1))
            .map_or(0, |(index, _)| index);
        head[window_start..]
            .rfind(char::is_whitespace)
            .map_or(head.len(), |offset| window_start + offset)
    };
    let mut quoted = head[..end].trim_end().to_string();
    quoted.push('…');
    Quoted {
        text: quoted,
        truncated: true,
    }
}

/// CTA label: « Voir plus » when a quote was cut, the email's own label otherwise.
pub fn cta_label(truncated: bool, own: LocalizedEmailText) -> LocalizedEmailText {
    if truncated {
        crate::email_i18n::text(SEE_MORE_CTA_KEY)
    } else {
        own
    }
}

/// Logs an email refused after the record was saved — the command still succeeds.
pub fn log_send_failure(message: &str, error: &PortakiError) {
    let mut fields = log::Fields::new();
    fields.insert("error", &error.to_string());
    let _ = log::warn(message, &fields);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_at_the_cap_is_quoted_as_is() {
        let text = "é".repeat(GUEST_TEXT_EMAIL_MAX_CHARS);
        assert_eq!(
            quote_guest_text(&text),
            Quoted {
                text: text.clone(),
                truncated: false
            }
        );
    }

    #[test]
    fn long_text_without_spaces_is_cut_at_the_cap_in_chars() {
        let quoted = quote_guest_text(&"é".repeat(20_000));
        assert!(quoted.truncated);
        assert_eq!(
            quoted.text,
            format!("{}…", "é".repeat(GUEST_TEXT_EMAIL_MAX_CHARS))
        );
    }

    #[test]
    fn the_cut_backs_off_to_a_word_boundary_within_the_window() {
        // Space at char 480, inside the last 40 chars before the cap.
        let text = format!("{} {}", "a".repeat(480), "b".repeat(100));
        assert_eq!(
            quote_guest_text(&text).text,
            format!("{}…", "a".repeat(480))
        );
    }

    #[test]
    fn a_space_before_the_window_is_ignored() {
        let text = format!("{} {}", "a".repeat(400), "b".repeat(200));
        let quoted = quote_guest_text(&text).text;
        assert_eq!(quoted.chars().count(), GUEST_TEXT_EMAIL_MAX_CHARS + 1);
    }

    #[test]
    fn a_cut_right_before_a_space_keeps_the_whole_cap() {
        let text = format!(
            "{} {}",
            "a".repeat(GUEST_TEXT_EMAIL_MAX_CHARS),
            "b".repeat(10)
        );
        assert_eq!(
            quote_guest_text(&text).text,
            format!("{}…", "a".repeat(GUEST_TEXT_EMAIL_MAX_CHARS))
        );
    }
}
