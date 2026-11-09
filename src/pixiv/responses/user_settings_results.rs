use crate::enums::Restrict;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserFollowDetailResult {
    pub follow_detail: UserFollowDetail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserFollowDetail {
    pub is_followed: bool,
    /// `None` while the user is not followed; Pixiv sends an empty string.
    #[serde(default, deserialize_with = "deserialize_follow_restrict")]
    pub restrict: Option<Restrict>,
}

fn deserialize_follow_restrict<'de, D>(deserializer: D) -> Result<Option<Restrict>, D::Error>
where
    D: Deserializer<'de>,
{
    let text = Option::<String>::deserialize(deserializer)?;
    match text.as_deref() {
        None | Some("") => Ok(None),
        Some("public") => Ok(Some(Restrict::Public)),
        Some("private") => Ok(Some(Restrict::Private)),
        Some(other) => Err(serde::de::Error::custom(format!(
            "unknown follow restrict: {other}"
        ))),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiShowSettingsResult {
    pub show_ai: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfilePresetsResult {
    pub profile_presets: ProfilePresets,
}

/// The choices of the profile form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfilePresets {
    pub addresses: Vec<ProfileAddressPreset>,
    pub countries: Vec<ProfileCountryPreset>,
    pub jobs: Vec<ProfileJobPreset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileAddressPreset {
    pub id: u64,
    pub name: String,
    /// A country has to be chosen as well.
    pub is_global: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileCountryPreset {
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileJobPreset {
    pub id: u64,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unfollowed_user_has_no_restrict() {
        let result: UserFollowDetailResult =
            serde_json::from_str(r#"{"follow_detail":{"is_followed":false,"restrict":""}}"#)
                .unwrap();
        assert!(!result.follow_detail.is_followed);
        assert_eq!(result.follow_detail.restrict, None);
    }

    #[test]
    fn a_followed_user_reads_its_restrict() {
        let result: UserFollowDetailResult =
            serde_json::from_str(r#"{"follow_detail":{"is_followed":true,"restrict":"private"}}"#)
                .unwrap();
        assert_eq!(result.follow_detail.restrict, Some(Restrict::Private));
    }

    #[test]
    fn an_unknown_restrict_is_an_error() {
        assert!(
            serde_json::from_str::<UserFollowDetailResult>(
                r#"{"follow_detail":{"is_followed":true,"restrict":"friends"}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn presets_ignore_the_default_images() {
        let result: ProfilePresetsResult = serde_json::from_str(
            r#"{"profile_presets":{"addresses":[{"id":1,"name":"a","is_global":true}],"countries":[{"code":"JP","name":"b"}],"jobs":[{"id":2,"name":"c"}],"default_profile_image_urls":{"medium":"x"}}}"#,
        )
        .unwrap();
        assert!(result.profile_presets.addresses[0].is_global);
        assert_eq!(result.profile_presets.countries[0].code, "JP");
        assert_eq!(result.profile_presets.jobs[0].id, 2);
    }
}
