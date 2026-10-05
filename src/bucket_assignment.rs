use serde::Serialize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Variant { Control, Treatment }

pub fn assign(subject: &str, remote_value: Option<&str>) -> Variant {
    if let Some(value) = remote_value {
        if value.eq_ignore_ascii_case("treatment") { return Variant::Treatment; }
        if value.eq_ignore_ascii_case("control") { return Variant::Control; }
    }
    let mut hasher = DefaultHasher::new();
    subject.hash(&mut hasher);
    if hasher.finish() % 2 == 0 { Variant::Control } else { Variant::Treatment }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_subscriber_stays_in_one_bucket() {
        assert_eq!(assign("creator-checkout-alice", None), assign("creator-checkout-alice", None));
    }
    #[test]
    fn flag_value_can_select_treatment() {
        assert_eq!(assign("any", Some("treatment")), Variant::Treatment);
    }
}

