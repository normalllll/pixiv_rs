use super::*;

/// Every field Pixiv's profile form submits. Publicity and the profile image
/// are not part of this request and keep their server values.
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
        let mut fields = params!([
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
            &mut fields,
            "country",
            edit.country_code.filter(|code| !code.is_empty()),
        );
        self.request_text(
            Endpoint::AppApi,
            Method::POST,
            "/v1/user/profile/edit",
            Vec::new(),
            RequestBody::Multipart(fields),
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
