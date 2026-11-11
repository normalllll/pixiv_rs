use super::*;

/// A new profile image sent along with the profile edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileImageUpload {
    pub bytes: Vec<u8>,
    pub file_name: String,
    /// For example `image/jpeg`.
    pub mime: String,
}

/// Every field Pixiv's profile form submits. Publicity is not part of this
/// request and keeps its server value; the profile image is replaced only
/// when `profile_image` is set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserProfileEdit {
    pub user_name: String,
    /// `male`, `female` or `unknown`.
    pub gender: String,
    pub address_id: u64,
    /// Required only for addresses Pixiv marks as global.
    pub country_code: Option<String>,
    pub job_id: u64,
    pub webpage: String,
    pub twitter: String,
    pub comment: String,
    /// `YYYY-MM-DD`, or empty when unset.
    pub birthday: String,
    pub profile_image: Option<ProfileImageUpload>,
}

/// The whole workspace form; a field left empty is cleared on Pixiv.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserWorkspaceEdit {
    pub pc: String,
    pub monitor: String,
    pub tool: String,
    pub scanner: String,
    pub tablet: String,
    pub mouse: String,
    pub printer: String,
    pub desktop: String,
    pub music: String,
    pub desk: String,
    pub chair: String,
    pub comment: String,
}

impl PixivApi {
    pub async fn get_user_follow_detail(
        &self,
        user_id: u64,
    ) -> Result<UserFollowDetailResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/user/follow/detail",
            params!([("user_id", user_id)]),
        )
        .await
    }

    pub async fn get_profile_presets(&self) -> Result<ProfilePresetsResult, PixivError> {
        self.get_json(Endpoint::AppApi, "/v1/user/profile/presets", Vec::new())
            .await
    }

    pub async fn post_user_profile_edit(
        &self,
        edit: UserProfileEdit,
    ) -> Result<String, PixivError> {
        self.request_text(
            Endpoint::AppApi,
            Method::POST,
            "/v1/user/profile/edit",
            Vec::new(),
            RequestBody::Multipart(profile_edit_fields(edit)),
        )
        .await
    }

    pub async fn post_user_workspace_edit(
        &self,
        edit: UserWorkspaceEdit,
    ) -> Result<String, PixivError> {
        self.request_text(
            Endpoint::AppApi,
            Method::POST,
            "/v1/user/workspace/edit",
            Vec::new(),
            RequestBody::Form(params!([
                ("pc", edit.pc),
                ("monitor", edit.monitor),
                ("tool", edit.tool),
                ("scanner", edit.scanner),
                ("tablet", edit.tablet),
                ("mouse", edit.mouse),
                ("printer", edit.printer),
                ("desktop", edit.desktop),
                ("music", edit.music),
                ("desk", edit.desk),
                ("chair", edit.chair),
                ("comment", edit.comment),
            ])),
        )
        .await
    }
}

fn profile_edit_fields(edit: UserProfileEdit) -> Vec<MultipartField> {
    let mut texts = params!([
        ("user_name", edit.user_name),
        ("gender", edit.gender),
        ("address", edit.address_id),
        ("job", edit.job_id),
        ("webpage", edit.webpage),
        ("twitter", edit.twitter),
        ("comment", edit.comment),
        ("birthday", edit.birthday),
    ]);
    push_optional(
        &mut texts,
        "country",
        edit.country_code.filter(|code| !code.is_empty()),
    );
    let mut fields: Vec<MultipartField> = texts
        .into_iter()
        .map(|(name, value)| MultipartField::Text { name, value })
        .collect();
    if let Some(image) = edit.profile_image {
        fields.push(MultipartField::File {
            name: "profile_image".to_owned(),
            file_name: image.file_name,
            mime: image.mime,
            bytes: image.bytes,
        });
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(profile_image: Option<ProfileImageUpload>) -> UserProfileEdit {
        UserProfileEdit {
            user_name: "name".into(),
            gender: "unknown".into(),
            address_id: 7,
            country_code: Some("JP".into()),
            job_id: 3,
            webpage: "w".into(),
            twitter: "t".into(),
            comment: "c".into(),
            birthday: "2000-01-02".into(),
            profile_image,
        }
    }

    fn text(name: &str, value: &str) -> MultipartField {
        MultipartField::Text {
            name: name.to_owned(),
            value: value.to_owned(),
        }
    }

    fn today() -> Vec<MultipartField> {
        vec![
            text("user_name", "name"),
            text("gender", "unknown"),
            text("address", "7"),
            text("job", "3"),
            text("webpage", "w"),
            text("twitter", "t"),
            text("comment", "c"),
            text("birthday", "2000-01-02"),
            text("country", "JP"),
        ]
    }

    #[test]
    fn without_an_image_the_fields_are_the_text_fields_only() {
        assert_eq!(profile_edit_fields(edit(None)), today());
    }

    #[test]
    fn an_empty_country_is_left_out() {
        let mut edit = edit(None);
        edit.country_code = Some(String::new());
        let mut expected = today();
        expected.pop();
        assert_eq!(profile_edit_fields(edit), expected);
    }

    #[test]
    fn an_image_is_the_last_field() {
        let image = ProfileImageUpload {
            bytes: vec![1, 2, 3],
            file_name: "a.jpg".into(),
            mime: "image/jpeg".into(),
        };
        let fields = profile_edit_fields(edit(Some(image)));
        let mut expected = today();
        expected.push(MultipartField::File {
            name: "profile_image".into(),
            file_name: "a.jpg".into(),
            mime: "image/jpeg".into(),
            bytes: vec![1, 2, 3],
        });
        assert_eq!(fields, expected);
    }
}
