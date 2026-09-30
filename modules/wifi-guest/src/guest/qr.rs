//! The `WIFI:` payload a phone camera turns into a joined network (§2.2).
//!
//! A guest arrives with luggage in one hand and a phone in the other, and typing `Villa-Guest_2026!`
//! on a phone keyboard in a dim hallway is the worst minute of an arrival. The code removes it.
//!
//! The scheme is the de-facto one every phone camera reads:
//!
//! ```text
//! WIFI:T:WPA;S:Villa-Guest;P:soleil2026;;
//! ```
//!
//! Its separators are `;` and `:`, so a network name or a password containing either has to be
//! escaped or the code silently means something else — `S:Café;Bar` announces a network called
//! `Café` and a stray field. Hosts do use `;` and `:` in passwords, so this is not a theoretical
//! case, and it fails in the one way we cannot afford: the code scans, the phone reports a failure
//! the guest cannot explain, and the booklet looks broken.

use crate::config::WifiSecurity;

/// The characters the scheme reserves, each escaped with a backslash.
fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(character, '\\' | ';' | ',' | ':' | '"') {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

/// The payload for this network, or `None` when there is nothing a phone could join.
///
/// `None` on an empty network name: a code without `S:` joins nothing, and showing one would be
/// worse than showing none — the guest scans, waits, and gets nothing to explain it.
///
/// An open network carries no `P:` at all rather than an empty one, which is what `nopass` means.
pub fn wifi_payload(ssid: &str, password: &str, security: WifiSecurity) -> Option<String> {
    let ssid = ssid.trim();
    if ssid.is_empty() {
        return None;
    }

    let password = password.trim();
    // An open network, whether the host said so or simply left the password empty. Trusting the
    // declared type alone would hand the phone a WPA code with no key to offer.
    let open = matches!(security, WifiSecurity::Nopass) || password.is_empty();
    if open {
        return Some(format!("WIFI:T:nopass;S:{};;", escape(ssid)));
    }

    Some(format!(
        "WIFI:T:{};S:{};P:{};;",
        security.as_qr_type(),
        escape(ssid),
        escape(password)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wpa_network_carries_its_type_name_and_key() {
        assert_eq!(
            wifi_payload("Villa-Guest", "soleil2026", WifiSecurity::Wpa).unwrap(),
            "WIFI:T:WPA;S:Villa-Guest;P:soleil2026;;"
        );
    }

    #[test]
    fn wep_is_said_as_wep() {
        assert_eq!(
            wifi_payload("Vieux", "abc", WifiSecurity::Wep).unwrap(),
            "WIFI:T:WEP;S:Vieux;P:abc;;"
        );
    }

    #[test]
    fn an_open_network_carries_no_key_field_at_all() {
        assert_eq!(
            wifi_payload("Portail", "", WifiSecurity::Wpa).unwrap(),
            "WIFI:T:nopass;S:Portail;;"
        );
        assert_eq!(
            wifi_payload("Portail", "ignoré", WifiSecurity::Nopass).unwrap(),
            "WIFI:T:nopass;S:Portail;;"
        );
    }

    #[test]
    fn the_reserved_characters_are_escaped_in_both_fields() {
        // Without this, `Café;Bar` would end the field and the rest would be read as another.
        assert_eq!(
            wifi_payload("Café;Bar", "a:b,c\\d\"e", WifiSecurity::Wpa).unwrap(),
            "WIFI:T:WPA;S:Café\\;Bar;P:a\\:b\\,c\\\\d\\\"e;;"
        );
    }

    #[test]
    fn a_network_without_a_name_has_no_code() {
        assert_eq!(wifi_payload("", "soleil", WifiSecurity::Wpa), None);
        assert_eq!(wifi_payload("   ", "soleil", WifiSecurity::Wpa), None);
    }

    #[test]
    fn surrounding_space_is_dropped_before_escaping() {
        assert_eq!(
            wifi_payload("  Villa  ", "  clé  ", WifiSecurity::Wpa).unwrap(),
            "WIFI:T:WPA;S:Villa;P:clé;;"
        );
    }
}
