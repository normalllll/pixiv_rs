use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct LiveConfig {
    account: UserAccountResult,
    proxy: Option<String>,
    #[serde(default)]
    accept_invalid_certs: bool,
}

fn live_api() -> PixivApi {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".local/session.json");
    let text = std::fs::read_to_string(path).expect("Create ignored .local/session.json first");
    let session: LiveConfig =
        serde_json::from_str(&text).unwrap_or_else(|_| panic!("Invalid local session format"));
    let mut config = PixivApiConfig::new(
        String::new(),
        "en-US".into(),
        "api-tests".into(),
        Some(session.account),
        session.accept_invalid_certs,
    );
    config.proxy = session.proxy;
    PixivApi::new(config)
}

async fn checked<T>(result: Result<T, PixivError>, label: &str) -> T {
    // Never print response bodies, account data, tags, or URLs from private lists.
    let value = result.unwrap_or_else(|e| panic!("{label}: {:?}, status {:?}", e.kind, e.status));
    println!("PASS {label}");
    tokio::time::sleep(Duration::from_millis(700)).await;
    value
}

/// Reads the refresh token from a freepiv Hive box. The box is an append-only
/// log, so the last occurrence of the key is the current value.
fn freepiv_refresh_token() -> String {
    let path = std::env::var("FREEPIV_APP_SETTINGS_HIVE")
        .expect("Set FREEPIV_APP_SETTINGS_HIVE to freepiv's app_settings.hive");
    let bytes = std::fs::read(path).expect("Read freepiv settings box");
    const HIVE_STRING_TYPE: u8 = 4;
    for key in [b"refresh_token".as_slice(), b"refreshToken".as_slice()] {
        let Some(start) = bytes.windows(key.len()).rposition(|window| window == key) else {
            continue;
        };
        let value = &bytes[start + key.len()..];
        assert_eq!(value[0], HIVE_STRING_TYPE, "Unexpected Hive value type");
        let length = u32::from_le_bytes(value[1..5].try_into().unwrap()) as usize;
        return String::from_utf8(value[5..5 + length].to_vec()).expect("UTF-8 refresh token");
    }
    panic!("No stored account in freepiv settings box");
}

async fn freepiv_api() -> PixivApi {
    let api = PixivApi::new(PixivApiConfig::new(
        String::new(),
        "en-US".into(),
        "api-tests".into(),
        None,
        false,
    ));
    let account = api
        .auth
        .refresh_auth_token(freepiv_refresh_token())
        .await
        .unwrap_or_else(|e| panic!("refresh: {:?}, status {:?}", e.kind, e.status));
    api.set_account(Some(account));
    api
}

/// Pixiv's own error text only; never the request or the rest of the body.
fn error_detail(e: &PixivError) -> String {
    let detail = e
        .body
        .as_deref()
        .and_then(|body| serde_json::from_str::<serde_json::Value>(body).ok())
        .and_then(|body| {
            let error = body.get("error")?;
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .filter(|m| !m.is_empty())
                .or_else(|| error.get("user_message").and_then(|m| m.as_str()))?;
            Some(message.chars().take(120).collect::<String>())
        });
    format!("{:?} {:?} {detail:?}", e.status, e.kind)
}

struct WriteRun {
    failures: Vec<String>,
}

impl WriteRun {
    fn check(&mut self, label: &str, ok: bool) -> bool {
        println!("{} {label}", if ok { "PASS" } else { "FAIL" });
        if !ok {
            self.failures.push(label.to_owned());
        }
        ok
    }

    async fn write(&mut self, label: &str, result: Result<String, PixivError>) -> bool {
        let ok = match result {
            Ok(_) => true,
            Err(e) => {
                println!("ERROR {label}: {}", error_detail(&e));
                false
            }
        };
        tokio::time::sleep(Duration::from_millis(1000)).await;
        self.check(label, ok)
    }
}

async fn raw_get(api: &PixivApi, path: &str, query: Params) -> Option<serde_json::Value> {
    let result = api
        .get_json::<serde_json::Value>(Endpoint::AppApi, path, query)
        .await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    match result {
        Ok(value) => Some(value),
        Err(e) => {
            println!("ERROR GET {path}: {}", error_detail(&e));
            None
        }
    }
}

fn find_key<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
    match value {
        serde_json::Value::Object(map) => map
            .get(key)
            .or_else(|| map.values().find_map(|child| find_key(child, key))),
        serde_json::Value::Array(items) => items.iter().find_map(|child| find_key(child, key)),
        _ => None,
    }
}

fn contains_id(value: &serde_json::Value, id: u64) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            map.get("id").and_then(|v| v.as_u64()) == Some(id)
                || map.values().any(|child| contains_id(child, id))
        }
        serde_json::Value::Array(items) => items.iter().any(|child| contains_id(child, id)),
        _ => false,
    }
}

fn list_items<'a>(value: &'a Option<serde_json::Value>, key: &str) -> Vec<&'a serde_json::Value> {
    value
        .as_ref()
        .and_then(|value| value.get(key))
        .and_then(|list| list.as_array())
        .map(|list| list.iter().collect())
        .unwrap_or_default()
}

fn unbookmarked_safe_id(items: &[&serde_json::Value]) -> Option<u64> {
    items
        .iter()
        .find(|item| {
            item["is_bookmarked"].as_bool() == Some(false) && item["x_restrict"].as_i64() == Some(0)
        })
        .and_then(|item| item["id"].as_u64())
}

fn series_ids(items: &[&serde_json::Value]) -> Vec<u64> {
    let mut ids: Vec<u64> = items
        .iter()
        .filter(|item| item["x_restrict"].as_i64() == Some(0))
        .filter_map(|item| item["series"]["id"].as_u64())
        .collect();
    ids.dedup();
    ids
}

const PIXIV_OFFICIAL_USER: u64 = 11;
const TEST_TAG_A: &str = "freepiv-live-test-a";
const TEST_TAG_B: &str = "freepiv-live-test-b";
const TEST_TEXT_MARKER: &str = "[freepiv-test]";

#[tokio::test]
#[ignore = "Changes a real Pixiv account and restores it; run explicitly"]
async fn live_public_bookmark_switch() {
    let api = freepiv_api().await;
    let mut run = WriteRun {
        failures: Vec::new(),
    };
    let user_id = api.account().unwrap().user.id.parse::<u64>().unwrap();
    let list = api
        .get_user_illust_bookmark_page_with_options(user_id, BookmarkPageOptions::default())
        .await
        .unwrap_or_else(|e| panic!("bookmarks: {}", error_detail(&e)));
    let head: Vec<_> = list.illusts.iter().take(3).map(|i| i.id).collect();
    let id = *head.first().expect("Public bookmark list is empty");
    println!("--- public/private switch: illust {id}");
    tokio::time::sleep(Duration::from_millis(1000)).await;

    let read = || async {
        let detail = api.get_illust_bookmark_detail(id).await;
        tokio::time::sleep(Duration::from_millis(1000)).await;
        detail.ok().map(|d| {
            let tags: Vec<String> = d
                .bookmark_detail
                .tags
                .iter()
                .filter(|tag| tag.is_registered)
                .map(|tag| tag.name.clone())
                .collect();
            (
                d.bookmark_detail.is_bookmarked,
                d.bookmark_detail.restrict,
                tags,
            )
        })
    };
    let original = read().await.expect("Read bookmark state");
    assert!(
        original.0 && original.1 == Restrict::Public,
        "First listed bookmark is not public"
    );
    let tags = original.2.clone();
    println!("INFO registered tag count: {}", tags.len());
    let options = |restrict| BookmarkAddOptions {
        tags: tags.clone(),
        restrict,
        is_novel: false,
    };

    if run
        .write(
            "bookmark switch to private",
            api.post_bookmark_add(id, options(Restrict::Private)).await,
        )
        .await
    {
        run.check(
            "bookmark is private with the same tags",
            read().await == Some((true, Restrict::Private, tags.clone())),
        );
    }
    let mut restored = run
        .write(
            "CLEANUP bookmark switch to public",
            api.post_bookmark_add(id, options(Restrict::Public)).await,
        )
        .await;
    if !restored {
        restored = run
            .write(
                "CLEANUP bookmark switch to public (retry)",
                api.post_bookmark_add(id, options(Restrict::Public)).await,
            )
            .await;
    }
    run.check(
        "RESTORED bookmark state",
        restored && read().await == Some(original.clone()),
    );

    tokio::time::sleep(Duration::from_millis(2000)).await;
    match api
        .get_user_illust_bookmark_page_with_options(user_id, BookmarkPageOptions::default())
        .await
    {
        Ok(after) => {
            let after: Vec<_> = after.illusts.iter().take(3).map(|i| i.id).collect();
            run.check("bookmark list order is unchanged", after == head);
        }
        Err(e) => println!("ERROR order check: {}", error_detail(&e)),
    }
    assert!(run.failures.is_empty(), "Failures: {:?}", run.failures);
}

fn comparable_detail(value: &serde_json::Value) -> serde_json::Value {
    let mut copy = value.clone();
    if let Some(profile) = copy.get_mut("profile").and_then(|p| p.as_object_mut()) {
        profile.retain(|key, _| !key.starts_with("total_"));
    }
    copy
}

fn changed_paths(
    a: &serde_json::Value,
    b: &serde_json::Value,
    path: &str,
    found: &mut Vec<String>,
) {
    match (a.as_object(), b.as_object()) {
        (Some(left), Some(right)) => {
            for key in left
                .keys()
                .chain(right.keys().filter(|k| !left.contains_key(*k)))
            {
                let null = serde_json::Value::Null;
                changed_paths(
                    left.get(key).unwrap_or(&null),
                    right.get(key).unwrap_or(&null),
                    &format!("{path}.{key}"),
                    found,
                );
            }
        }
        _ if a != b => found.push(path.to_owned()),
        _ => {}
    }
}

/// Field names that still differ from `expected` once Pixiv has settled.
async fn settled_diff(api: &PixivApi, id: u64, expected: &serde_json::Value) -> Vec<String> {
    let mut found = vec!["<unread>".to_owned()];
    for _ in 0..4 {
        if let Some(current) = raw_get(api, "/v1/user/detail", params!([("user_id", id)])).await {
            found.clear();
            changed_paths(expected, &comparable_detail(&current), "$", &mut found);
            if found.is_empty() {
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(1500)).await;
    }
    found
}

fn workspace_edit(workspace: &UserWorkspace) -> UserWorkspaceEdit {
    UserWorkspaceEdit {
        pc: workspace.pc.clone(),
        monitor: workspace.monitor.clone(),
        tool: workspace.tool.clone(),
        scanner: workspace.scanner.clone(),
        tablet: workspace.tablet.clone(),
        mouse: workspace.mouse.clone(),
        printer: workspace.printer.clone(),
        desktop: workspace.desktop.clone(),
        music: workspace.music.clone(),
        desk: workspace.desk.clone(),
        chair: workspace.chair.clone(),
        comment: workspace.comment.clone(),
    }
}

fn profile_edit(detail: &UserDetailResult) -> UserProfileEdit {
    let profile = &detail.profile;
    UserProfileEdit {
        user_name: detail.user.name.clone(),
        gender: if profile.gender.is_empty() {
            "unknown".to_owned()
        } else {
            profile.gender.clone()
        },
        address_id: profile.address_id,
        country_code: Some(profile.country_code.clone()),
        job_id: profile.job_id,
        webpage: profile.webpage.clone().unwrap_or_default(),
        twitter: profile.twitter_account.clone().unwrap_or_default(),
        comment: detail.user.comment.clone().unwrap_or_default(),
        birthday: profile.birth.clone(),
    }
}

fn report_diff(run: &mut WriteRun, label: &str, diff: &[String]) -> bool {
    if !diff.is_empty() {
        println!("DIFF {label}: {diff:?}");
    }
    run.check(label, diff.is_empty())
}

#[tokio::test]
#[ignore = "Changes a real Pixiv account and restores it; run explicitly"]
async fn live_workspace_edit() {
    let api = freepiv_api().await;
    let mut run = WriteRun {
        failures: Vec::new(),
    };
    let id = api.account().unwrap().user.id.parse::<u64>().unwrap();
    let typed = api
        .get_user_detail(id)
        .await
        .unwrap_or_else(|e| panic!("detail: {}", error_detail(&e)));
    let snapshot = save_snapshot(&api, id).await;

    let original = workspace_edit(&typed.workspace);
    let mut changed = original.clone();
    changed.comment.push_str(TEST_TEXT_MARKER);
    let mut expected = snapshot.clone();
    expected["workspace"]["comment"] = serde_json::Value::String(changed.comment.clone());

    if run
        .write(
            "workspace edit",
            api.post_user_workspace_edit(changed).await,
        )
        .await
    {
        let diff = settled_diff(&api, id, &expected).await;
        report_diff(&mut run, "workspace changed only the comment", &diff);
    }
    let mut restored = run
        .write(
            "CLEANUP workspace restore",
            api.post_user_workspace_edit(original.clone()).await,
        )
        .await;
    if !restored {
        restored = run
            .write(
                "CLEANUP workspace restore (retry)",
                api.post_user_workspace_edit(original).await,
            )
            .await;
    }
    let diff = settled_diff(&api, id, &snapshot).await;
    report_diff(&mut run, "RESTORED workspace and profile", &diff);
    finish_snapshot(restored && diff.is_empty());
    assert!(run.failures.is_empty(), "Failures: {:?}", run.failures);
}

#[tokio::test]
#[ignore = "Changes a real Pixiv account and restores it; run explicitly"]
async fn live_profile_edit() {
    let api = freepiv_api().await;
    let mut run = WriteRun {
        failures: Vec::new(),
    };
    let id = api.account().unwrap().user.id.parse::<u64>().unwrap();
    let typed = api
        .get_user_detail(id)
        .await
        .unwrap_or_else(|e| panic!("detail: {}", error_detail(&e)));
    let snapshot = save_snapshot(&api, id).await;
    let original = profile_edit(&typed);

    // Resubmitting identical values exposes any field this form would clear.
    let accepted = run
        .write(
            "profile resubmit unchanged",
            api.post_user_profile_edit(original.clone()).await,
        )
        .await;
    let unchanged = accepted && {
        let diff = settled_diff(&api, id, &snapshot).await;
        report_diff(&mut run, "profile resubmit changed nothing", &diff)
    };
    if !accepted {
        println!("STOP profile form was rejected; nothing was changed");
        finish_snapshot(settled_diff(&api, id, &snapshot).await.is_empty());
        panic!("Failures: {:?}", run.failures);
    }

    if unchanged {
        let mut changed = original.clone();
        changed.comment.push_str(TEST_TEXT_MARKER);
        let mut expected = snapshot.clone();
        expected["user"]["comment"] = serde_json::Value::String(changed.comment.clone());
        if run
            .write("profile edit", api.post_user_profile_edit(changed).await)
            .await
        {
            let diff = settled_diff(&api, id, &expected).await;
            report_diff(&mut run, "profile changed only the comment", &diff);
        }
    }

    let mut restored = run
        .write(
            "CLEANUP profile restore",
            api.post_user_profile_edit(original.clone()).await,
        )
        .await;
    if !restored {
        restored = run
            .write(
                "CLEANUP profile restore (retry)",
                api.post_user_profile_edit(original).await,
            )
            .await;
    }
    let diff = settled_diff(&api, id, &snapshot).await;
    report_diff(&mut run, "RESTORED profile and workspace", &diff);
    finish_snapshot(restored && diff.is_empty());
    assert!(run.failures.is_empty(), "Failures: {:?}", run.failures);
}

fn snapshot_path() -> String {
    std::env::var("FREEPIV_PROFILE_SNAPSHOT")
        .expect("Set FREEPIV_PROFILE_SNAPSHOT to a recovery file path")
}

/// Keeps the pre-test values on disk until the account is confirmed restored.
async fn save_snapshot(api: &PixivApi, id: u64) -> serde_json::Value {
    let raw = raw_get(api, "/v1/user/detail", params!([("user_id", id)]))
        .await
        .expect("Read current profile");
    std::fs::write(snapshot_path(), serde_json::to_vec_pretty(&raw).unwrap())
        .expect("Write recovery snapshot");
    comparable_detail(&raw)
}

fn finish_snapshot(restored: bool) {
    if restored {
        std::fs::remove_file(snapshot_path()).expect("Remove recovery snapshot");
        println!("INFO recovery snapshot removed");
    } else {
        println!("WARN account differs from snapshot; recovery file kept");
    }
}

async fn test_illust_bookmark(api: &PixivApi, run: &mut WriteRun, id: u64) {
    println!("--- private illust bookmark, tags, overwrite: illust {id}");
    let before = api.get_illust_bookmark_detail(id).await;
    if !matches!(&before, Ok(d) if !d.bookmark_detail.is_bookmarked) {
        println!("SKIP illust is not verifiably unbookmarked");
        return;
    }
    let options = |tag: &str| BookmarkAddOptions {
        tags: vec![tag.to_owned()],
        restrict: Restrict::Private,
        is_novel: false,
    };
    let registered = |detail: &WorkBookmarkDetailResult| -> Vec<String> {
        detail
            .bookmark_detail
            .tags
            .iter()
            .filter(|tag| tag.is_registered)
            .map(|tag| tag.name.clone())
            .collect()
    };
    if run
        .write(
            "illust bookmark add (private, tag A)",
            api.post_bookmark_add(id, options(TEST_TAG_A)).await,
        )
        .await
    {
        if let Ok(d) = api.get_illust_bookmark_detail(id).await {
            run.check(
                "illust bookmark is private with tag A",
                d.bookmark_detail.is_bookmarked
                    && d.bookmark_detail.restrict == Restrict::Private
                    && registered(&d) == [TEST_TAG_A],
            );
        }
        tokio::time::sleep(Duration::from_millis(1000)).await;
        run.write(
            "illust bookmark overwrite (tag B)",
            api.post_bookmark_add(id, options(TEST_TAG_B)).await,
        )
        .await;
        if let Ok(d) = api.get_illust_bookmark_detail(id).await {
            run.check(
                "illust bookmark tags were replaced",
                registered(&d) == [TEST_TAG_B],
            );
        }
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
    run.write(
        "CLEANUP illust bookmark delete",
        api.post_bookmark_delete(id, false).await,
    )
    .await;
    let after = api.get_illust_bookmark_detail(id).await;
    run.check(
        "RESTORED illust is unbookmarked",
        matches!(&after, Ok(d) if !d.bookmark_detail.is_bookmarked),
    );
}

async fn test_novel_bookmark(api: &PixivApi, run: &mut WriteRun, id: u64) {
    println!("--- private novel bookmark: novel {id}");
    let before = api.get_novel_bookmark_detail(id).await;
    if !matches!(&before, Ok(d) if !d.bookmark_detail.is_bookmarked) {
        println!("SKIP novel is not verifiably unbookmarked");
        return;
    }
    let options = BookmarkAddOptions {
        tags: Vec::new(),
        restrict: Restrict::Private,
        is_novel: true,
    };
    if run
        .write(
            "novel bookmark add (private)",
            api.post_bookmark_add(id, options).await,
        )
        .await
    {
        if let Ok(d) = api.get_novel_bookmark_detail(id).await {
            run.check(
                "novel bookmark is private",
                d.bookmark_detail.is_bookmarked && d.bookmark_detail.restrict == Restrict::Private,
            );
        }
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
    run.write(
        "CLEANUP novel bookmark delete",
        api.post_bookmark_delete(id, true).await,
    )
    .await;
    let after = api.get_novel_bookmark_detail(id).await;
    run.check(
        "RESTORED novel is unbookmarked",
        matches!(&after, Ok(d) if !d.bookmark_detail.is_bookmarked),
    );
}

async fn test_private_follow(api: &PixivApi, run: &mut WriteRun) {
    let id = PIXIV_OFFICIAL_USER;
    println!("--- private follow: user {id}");
    let before = api.get_user_detail(id).await;
    if !matches!(&before, Ok(d) if !d.user.is_followed) {
        println!("SKIP user is not verifiably unfollowed");
        return;
    }
    let detail_query = || params!([("user_id", id)]);
    if run
        .write(
            "follow add (private)",
            api.post_follow_add(id, Restrict::Private).await,
        )
        .await
    {
        let detail = raw_get(api, "/v1/user/follow/detail", detail_query()).await;
        let detail = detail.as_ref().and_then(|d| d.get("follow_detail"));
        run.check(
            "follow is private",
            detail.is_some_and(|d| {
                d["is_followed"].as_bool() == Some(true)
                    && d["restrict"].as_str() == Some("private")
            }),
        );
    }
    run.write("CLEANUP follow delete", api.post_follow_delete(id).await)
        .await;
    let after = api.get_user_detail(id).await;
    run.check(
        "RESTORED user is unfollowed",
        matches!(&after, Ok(d) if !d.user.is_followed),
    );
}

async fn manga_watched(api: &PixivApi, id: u64) -> Option<bool> {
    let result = api.get_illust_series_page(id).await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    result
        .ok()
        .map(|page| page.illust_series_detail.watchlist_added)
}

async fn novel_watched(api: &PixivApi, id: u64) -> Option<bool> {
    let result = api.get_novel_series_page(id).await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    result
        .ok()
        .map(|page| page.novel_series_detail.watchlist_added)
}

async fn test_watchlist(api: &PixivApi, run: &mut WriteRun, ids: &[u64], is_novel: bool) {
    let kind = if is_novel { "novel" } else { "manga" };
    let watched = |id| async move {
        if is_novel {
            novel_watched(api, id).await
        } else {
            manga_watched(api, id).await
        }
    };
    let mut target = None;
    for id in ids.iter().copied().take(5) {
        if watched(id).await == Some(false) {
            target = Some(id);
            break;
        }
    }
    let Some(id) = target else {
        println!("SKIP no unwatched {kind} series available");
        return;
    };
    println!("--- {kind} watchlist: series {id}");
    if run
        .write(
            &format!("{kind} watchlist add"),
            api.post_watchlist_add(id, is_novel).await,
        )
        .await
    {
        run.check(
            &format!("{kind} series is watched"),
            watched(id).await == Some(true),
        );
    }
    run.write(
        &format!("CLEANUP {kind} watchlist delete"),
        api.post_watchlist_delete(id, is_novel).await,
    )
    .await;
    run.check(
        &format!("RESTORED {kind} series is unwatched"),
        watched(id).await == Some(false),
    );
}

async fn test_novel_marker(api: &PixivApi, run: &mut WriteRun, id: u64) {
    println!("--- novel marker: novel {id}");
    let webview = api.get_webview_novel(id).await;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let markers = raw_get(api, "/v2/novel/markers", Vec::new()).await;
    let unmarked = matches!(&webview, Ok(novel) if novel.marker.is_none())
        && markers.as_ref().is_some_and(|m| !contains_id(m, id));
    if !unmarked {
        println!(
            "SKIP novel is not verifiably unmarked (webview ok: {}, listed: {:?})",
            webview.is_ok(),
            markers.as_ref().map(|m| contains_id(m, id))
        );
        return;
    }
    if run
        .write("novel marker add", api.post_novel_marker_add(id, 1).await)
        .await
    {
        let listed = raw_get(api, "/v2/novel/markers", Vec::new()).await;
        run.check(
            "novel marker is listed",
            listed.as_ref().is_some_and(|m| contains_id(m, id)),
        );
    }
    run.write(
        "CLEANUP novel marker delete",
        api.post_novel_marker_delete(id).await,
    )
    .await;
    let listed = raw_get(api, "/v2/novel/markers", Vec::new()).await;
    run.check(
        "RESTORED novel marker is gone",
        listed.as_ref().is_some_and(|m| !contains_id(m, id)),
    );
}

async fn test_ai_setting(api: &PixivApi, run: &mut WriteRun) {
    println!("--- AI show setting");
    let read = || async {
        raw_get(api, "/v1/user/ai-show-settings", Vec::new())
            .await
            .and_then(|value| find_key(&value, "show_ai").and_then(|v| v.as_bool()))
    };
    let Some(original) = read().await else {
        println!("SKIP current AI setting is unreadable");
        return;
    };
    let mode = |show: bool| {
        if show {
            SearchAiMode::Show
        } else {
            SearchAiMode::Hide
        }
    };
    println!("INFO original show_ai={original}");
    if run
        .write(
            "AI setting flip",
            api.post_ai_show_settings(mode(!original)).await,
        )
        .await
    {
        run.check("AI setting changed", read().await == Some(!original));
    }
    run.write(
        "CLEANUP AI setting restore",
        api.post_ai_show_settings(mode(original)).await,
    )
    .await;
    run.check("RESTORED AI setting", read().await == Some(original));
}

#[tokio::test]
#[ignore = "Uses a real Pixiv account and network; run explicitly"]
async fn live_test_bookmark_tags_are_gone() {
    let api = freepiv_api().await;
    let user_id = api.account().unwrap().user.id.parse::<u64>().unwrap();
    let mut page = api
        .get_bookmark_tag_page(
            user_id,
            BookmarkTagOptions {
                restrict: Restrict::Private,
                is_novel: false,
            },
        )
        .await
        .unwrap_or_else(|e| panic!("tags: {}", error_detail(&e)));
    let mut pages = 1;
    loop {
        let left = page
            .bookmark_tags
            .iter()
            .any(|tag| tag.name == TEST_TAG_A || tag.name == TEST_TAG_B);
        assert!(!left, "Test tag is still listed");
        let Some(url) = page.next_url.clone() else {
            break;
        };
        tokio::time::sleep(Duration::from_millis(1000)).await;
        page = api
            .get_next_bookmark_tag_page(url)
            .await
            .unwrap_or_else(|e| panic!("tags: {}", error_detail(&e)));
        pages += 1;
    }
    println!("PASS no test tags across {pages} page(s)");
}

#[tokio::test]
#[ignore = "Changes a real Pixiv account and restores it; run explicitly"]
async fn live_illust_bookmark_tags() {
    let api = freepiv_api().await;
    let mut run = WriteRun {
        failures: Vec::new(),
    };
    let official = raw_get(
        &api,
        "/v1/user/illusts",
        params!([
            ("user_id", PIXIV_OFFICIAL_USER),
            ("type", "illust"),
            ("filter", "for_android"),
        ]),
    )
    .await;
    let id = unbookmarked_safe_id(&list_items(&official, "illusts"))
        .expect("No unbookmarked official illust");
    test_illust_bookmark(&api, &mut run, id).await;
    assert!(run.failures.is_empty(), "Failures: {:?}", run.failures);
}

#[tokio::test]
#[ignore = "Changes a real Pixiv account and restores it; run explicitly"]
async fn live_reversible_writes() {
    let api = freepiv_api().await;
    let mut run = WriteRun {
        failures: Vec::new(),
    };

    let official = raw_get(
        &api,
        "/v1/user/illusts",
        params!([
            ("user_id", PIXIV_OFFICIAL_USER),
            ("type", "illust"),
            ("filter", "for_android"),
        ]),
    )
    .await;
    let manga = raw_get(
        &api,
        "/v1/illust/ranking",
        params!([("filter", "for_android"), ("mode", "day_manga")]),
    )
    .await;
    let novels = raw_get(
        &api,
        "/v1/novel/ranking",
        params!([("filter", "for_android"), ("mode", "day")]),
    )
    .await;

    match unbookmarked_safe_id(&list_items(&official, "illusts")) {
        Some(id) => test_illust_bookmark(&api, &mut run, id).await,
        None => println!("SKIP no unbookmarked official illust"),
    }
    let novel_items = list_items(&novels, "novels");
    match unbookmarked_safe_id(&novel_items) {
        Some(id) => {
            test_novel_bookmark(&api, &mut run, id).await;
            test_novel_marker(&api, &mut run, id).await;
        }
        None => println!("SKIP no unbookmarked ranking novel"),
    }
    test_private_follow(&api, &mut run).await;
    test_watchlist(
        &api,
        &mut run,
        &series_ids(&list_items(&manga, "illusts")),
        false,
    )
    .await;
    test_watchlist(&api, &mut run, &series_ids(&novel_items), true).await;
    test_ai_setting(&api, &mut run).await;

    let user_id = api.account().unwrap().user.id.parse::<u64>().unwrap();
    if let Ok(tags) = api
        .get_bookmark_tag_page(
            user_id,
            BookmarkTagOptions {
                restrict: Restrict::Private,
                is_novel: false,
            },
        )
        .await
    {
        let left = tags
            .bookmark_tags
            .iter()
            .any(|tag| tag.name == TEST_TAG_A || tag.name == TEST_TAG_B);
        println!("INFO test tags still listed on first page: {left}");
    }

    assert!(run.failures.is_empty(), "Failures: {:?}", run.failures);
}

/// The shape of a response: top-level keys, list lengths and the keys of the
/// first item. Never the values.
fn print_shape(label: &str, value: &serde_json::Value) {
    let Some(map) = value.as_object() else {
        println!("SHAPE {label}: not an object");
        return;
    };
    println!("SHAPE {label}: keys={:?}", map.keys().collect::<Vec<_>>());
    for (key, child) in map {
        if let Some(items) = child.as_array() {
            let first = items
                .first()
                .and_then(|item| item.as_object())
                .map(|item| item.keys().cloned().collect::<Vec<_>>());
            println!("SHAPE {label}.{key}: len={} first={first:?}", items.len());
        } else if let Some(inner) = child.as_object() {
            println!(
                "SHAPE {label}.{key}: keys={:?}",
                inner.keys().collect::<Vec<_>>()
            );
            for (name, nested) in inner {
                if let Some(items) = nested.as_array() {
                    let first = items
                        .first()
                        .and_then(|item| item.as_object())
                        .map(|item| item.keys().cloned().collect::<Vec<_>>());
                    println!(
                        "SHAPE {label}.{key}.{name}: len={} first={first:?}",
                        items.len()
                    );
                }
            }
        }
    }
}

async fn follow_restrict(api: &PixivApi, id: u64) -> Option<String> {
    let detail = raw_get(api, "/v1/user/follow/detail", params!([("user_id", id)])).await?;
    let detail = detail.get("follow_detail")?;
    Some(format!(
        "followed={} restrict={}",
        detail["is_followed"], detail["restrict"]
    ))
}

async fn probe_follow_switch(api: &PixivApi, run: &mut WriteRun) {
    let id = PIXIV_OFFICIAL_USER;
    println!("--- follow switch: user {id}");
    let before = api.get_user_detail(id).await;
    if !matches!(&before, Ok(d) if !d.user.is_followed) {
        println!("SKIP user is not verifiably unfollowed");
        return;
    }
    let expect = |state: Option<String>, restrict: &str| {
        println!("INFO state after {restrict}: {state:?}");
        run_expect(state, restrict)
    };
    let mut results = Vec::new();
    for restrict in [Restrict::Public, Restrict::Private, Restrict::Public] {
        let name = format!("{restrict:?}").to_lowercase();
        let result = api.post_follow_add(id, restrict).await;
        tokio::time::sleep(Duration::from_millis(1000)).await;
        if let Err(e) = &result {
            println!("ERROR follow add {name}: {}", error_detail(e));
        }
        let state = follow_restrict(api, id).await;
        results.push((name.clone(), expect(state, &name)));
    }
    for (name, ok) in results {
        run.check(&format!("follow reads back as {name}"), ok);
    }
    run.write("CLEANUP follow delete", api.post_follow_delete(id).await)
        .await;
    let after = api.get_user_detail(id).await;
    run.check(
        "RESTORED user is unfollowed",
        matches!(&after, Ok(d) if !d.user.is_followed),
    );
}

fn run_expect(state: Option<String>, restrict: &str) -> bool {
    state.is_some_and(|state| {
        state.contains("followed=true") && state.contains(&format!("restrict=\"{restrict}\""))
    })
}

const WATCHLIST_ROUTES: [(&str, bool); 4] = [
    ("/v1/watchlist/manga", false),
    ("/v2/watchlist/manga", false),
    ("/v1/watchlist/novel", true),
    ("/v2/watchlist/novel", true),
];

async fn probe_watchlist_reads(api: &PixivApi, run: &mut WriteRun) {
    let manga = raw_get(
        api,
        "/v1/illust/ranking",
        params!([("filter", "for_android"), ("mode", "day_manga")]),
    )
    .await;
    let novels = raw_get(
        api,
        "/v1/novel/ranking",
        params!([("filter", "for_android"), ("mode", "day")]),
    )
    .await;
    let manga_ids = series_ids(&list_items(&manga, "illusts"));
    let novel_ids = series_ids(&list_items(&novels, "novels"));

    for (route, is_novel) in WATCHLIST_ROUTES {
        println!("--- watchlist read: {route}");
        let query = || params!([("filter", "for_android")]);
        let Some(listed) = raw_get(api, route, query()).await else {
            continue;
        };
        print_shape(route, &listed);

        let ids = if is_novel { &novel_ids } else { &manga_ids };
        let mut target = None;
        for id in ids.iter().copied().take(5) {
            let watched = if is_novel {
                novel_watched(api, id).await
            } else {
                manga_watched(api, id).await
            };
            if watched == Some(false) {
                target = Some(id);
                break;
            }
        }
        let Some(id) = target else {
            println!("SKIP no unwatched series for {route}");
            continue;
        };
        if run
            .write(
                &format!("{route} add series"),
                api.post_watchlist_add(id, is_novel).await,
            )
            .await
        {
            let listed = raw_get(api, route, query()).await;
            let found = listed.as_ref().is_some_and(|value| contains_id(value, id));
            println!("INFO {route} lists the added series: {found}");
            if let Some(value) = &listed {
                print_shape(route, value);
            }
        }
        run.write(
            &format!("CLEANUP {route} delete series"),
            api.post_watchlist_delete(id, is_novel).await,
        )
        .await;
        let watched = if is_novel {
            novel_watched(api, id).await
        } else {
            manga_watched(api, id).await
        };
        run.check(
            &format!("RESTORED {route} series unwatched"),
            watched == Some(false),
        );
    }
    let control = raw_get(api, "/v1/watchlist/route-control", Vec::new()).await;
    println!("INFO control route answered: {}", control.is_some());
}

fn json_type(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(text) if text.is_empty() => "empty string",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Field names and JSON types, one level per object; never the values.
fn print_types(label: &str, value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            let types: Vec<_> = map
                .iter()
                .map(|(key, child)| format!("{key}:{}", json_type(child)))
                .collect();
            println!("TYPES {label}: {types:?}");
            for (key, child) in map {
                if child.is_object() || child.is_array() {
                    print_types(&format!("{label}.{key}"), child);
                }
            }
        }
        serde_json::Value::Array(items) => {
            if let Some(first) = items.first() {
                print_types(&format!("{label}[]"), first);
            }
        }
        _ => {}
    }
}

#[tokio::test]
#[ignore = "Uses a real Pixiv account and network; run explicitly"]
async fn live_account_settings_reads() {
    let api = freepiv_api().await;
    checked(
        api.get_user_follow_detail(PIXIV_OFFICIAL_USER).await,
        "follow detail",
    )
    .await;
    checked(api.get_ai_show_settings().await, "AI settings").await;
    let presets = checked(api.get_profile_presets().await, "profile presets").await;
    assert!(!presets.profile_presets.addresses.is_empty());
    assert!(!presets.profile_presets.jobs.is_empty());
    checked(api.get_watchlist_page(false).await, "manga watchlist").await;
    checked(api.get_watchlist_page(true).await, "novel watchlist").await;
}

#[tokio::test]
#[ignore = "Changes a real Pixiv account and restores it; run explicitly"]
async fn live_response_type_probe() {
    let api = freepiv_api().await;
    let mut run = WriteRun {
        failures: Vec::new(),
    };

    let before = api.get_user_detail(PIXIV_OFFICIAL_USER).await;
    if matches!(&before, Ok(d) if !d.user.is_followed)
        && let Some(detail) = raw_get(
            &api,
            "/v1/user/follow/detail",
            params!([("user_id", PIXIV_OFFICIAL_USER)]),
        )
        .await
    {
        print_types("follow detail (unfollowed)", &detail);
        println!(
            "INFO unfollowed restrict value: {}",
            detail["follow_detail"]["restrict"]
        );
    }
    if let Some(settings) = raw_get(&api, "/v1/user/ai-show-settings", Vec::new()).await {
        print_types("ai settings", &settings);
    }

    let novels = raw_get(
        &api,
        "/v1/novel/ranking",
        params!([("filter", "for_android"), ("mode", "day")]),
    )
    .await;
    let manga = raw_get(
        &api,
        "/v1/illust/ranking",
        params!([("filter", "for_android"), ("mode", "day_manga")]),
    )
    .await;

    let mut novel = None;
    for id in list_items(&novels, "novels")
        .iter()
        .filter(|item| item["x_restrict"].as_i64() == Some(0))
        .filter_map(|item| item["id"].as_u64())
        .take(15)
    {
        let webview = api.get_webview_novel(id).await;
        tokio::time::sleep(Duration::from_millis(1000)).await;
        if matches!(&webview, Ok(text) if text.marker.is_none() && text.text.contains("[newpage]"))
        {
            novel = Some(id);
            break;
        }
    }
    if let Some(id) = novel {
        {
            if run
                .write("marker add page 2", api.post_novel_marker_add(id, 2).await)
                .await
            {
                let marked = api.get_webview_novel(id).await;
                tokio::time::sleep(Duration::from_millis(1000)).await;
                println!(
                    "INFO webview marker after page 2: {:?}",
                    marked.ok().and_then(|novel| novel.marker)
                );
                if let Some(listed) = raw_get(&api, "/v2/novel/markers", Vec::new()).await {
                    print_types("markers", &listed);
                }
            }
            run.write(
                "CLEANUP marker delete",
                api.post_novel_marker_delete(id).await,
            )
            .await;
            let after = api.get_webview_novel(id).await;
            tokio::time::sleep(Duration::from_millis(1000)).await;
            run.check(
                "RESTORED marker is gone",
                matches!(&after, Ok(novel) if novel.marker.is_none()),
            );
        }
    } else {
        println!("SKIP no unmarked ranking novel with several pages");
    }

    let mut target = None;
    for id in series_ids(&list_items(&manga, "illusts"))
        .into_iter()
        .take(5)
    {
        if manga_watched(&api, id).await == Some(false) {
            target = Some(id);
            break;
        }
    }
    if let Some(id) = target {
        if run
            .write("watchlist add", api.post_watchlist_add(id, false).await)
            .await
            && let Some(listed) = raw_get(
                &api,
                "/v1/watchlist/manga",
                params!([("filter", "for_android")]),
            )
            .await
        {
            print_types("watchlist", &listed);
        }
        run.write(
            "CLEANUP watchlist delete",
            api.post_watchlist_delete(id, false).await,
        )
        .await;
        run.check(
            "RESTORED series unwatched",
            manga_watched(&api, id).await == Some(false),
        );
    }

    assert!(run.failures.is_empty(), "Failures: {:?}", run.failures);
}

#[tokio::test]
#[ignore = "Changes a real Pixiv account and restores it; run explicitly"]
async fn live_unverified_route_probe() {
    let api = freepiv_api().await;
    let mut run = WriteRun {
        failures: Vec::new(),
    };

    if let Some(presets) = raw_get(&api, "/v1/user/profile/presets", Vec::new()).await {
        print_shape("presets", &presets);
    }
    probe_follow_switch(&api, &mut run).await;
    probe_watchlist_reads(&api, &mut run).await;

    assert!(run.failures.is_empty(), "Failures: {:?}", run.failures);
}

#[tokio::test]
#[ignore = "Uses a real Pixiv account and network; run explicitly"]
async fn live_read_only() {
    let api = live_api();
    let user_id = api.account().unwrap().user.id.parse::<u64>().unwrap();
    checked(api.get_user_detail(user_id).await, "session validation").await;
    checked(api.get_mypixiv_user_page(user_id).await, "MyPixiv friends").await;
    for restrict in [Restrict::Public, Restrict::Private] {
        let options = BookmarkPageOptions {
            restrict,
            ..Default::default()
        };
        let illusts = checked(
            api.get_user_illust_bookmark_page_with_options(user_id, options.clone())
                .await,
            "illustration bookmarks",
        )
        .await;
        if let Some(url) = illusts.next_url {
            checked(
                api.get_next_illust_page(url).await,
                "illustration bookmark pagination",
            )
            .await;
        }
        let novels = checked(
            api.get_user_novel_bookmark_page_with_options(user_id, options)
                .await,
            "novel bookmarks",
        )
        .await;
        if let Some(url) = novels.next_url {
            checked(
                api.get_next_novel_page(url).await,
                "novel bookmark pagination",
            )
            .await;
        }
        for is_novel in [false, true] {
            let tags = checked(
                api.get_bookmark_tag_page(user_id, BookmarkTagOptions { restrict, is_novel })
                    .await,
                "bookmark tags",
            )
            .await;
            let no_tags = tags.bookmark_tags.is_empty();
            let tag = tags
                .bookmark_tags
                .first()
                .map(|tag| tag.name.clone())
                .unwrap_or_else(|| "__pixiv_rs_live_test_no_match__".into());
            let options = BookmarkPageOptions {
                restrict,
                tag: Some(tag),
                max_bookmark_id: None,
            };
            if is_novel {
                let page = checked(
                    api.get_user_novel_bookmark_page_with_options(user_id, options)
                        .await,
                    "novel tag filter",
                )
                .await;
                if no_tags {
                    assert!(
                        page.novels.is_empty(),
                        "Unknown bookmark tag returned results"
                    );
                }
            } else {
                let page = checked(
                    api.get_user_illust_bookmark_page_with_options(user_id, options)
                        .await,
                    "illustration tag filter",
                )
                .await;
                if no_tags {
                    assert!(
                        page.illusts.is_empty(),
                        "Unknown bookmark tag returned results"
                    );
                }
            }
        }
    }
    // Fixed past date avoids requesting today's ranking before publication.
    let date = "2026-09-10".to_owned();
    let illusts = checked(
        api.get_illust_ranking_page_on_date(IllustRankingMode::Day, date.clone())
            .await,
        "historical illustration ranking",
    )
    .await;
    let historical_manga = checked(
        api.get_manga_ranking_page_on_date(MangaRankingMode::Day, date.clone())
            .await,
        "historical manga ranking",
    )
    .await;
    let novels = checked(
        api.get_novel_ranking_page_on_date(NovelRankingMode::Day, date)
            .await,
        "historical novel ranking",
    )
    .await;
    assert!(
        !historical_manga.illusts.is_empty(),
        "Historical manga ranking is empty"
    );
    assert!(!illusts.illusts.is_empty(), "Historical ranking is empty");
    checked(
        api.get_user_related_page(0, illusts.illusts[0].user.id)
            .await,
        "related users",
    )
    .await;
    let illust_id = illusts.illusts[0].id;
    let novel_id = novels
        .novels
        .first()
        .expect("Historical novel ranking is empty")
        .id;
    checked(
        api.get_illust_bookmark_detail(illust_id).await,
        "illustration bookmark detail",
    )
    .await;
    checked(
        api.get_novel_bookmark_detail(novel_id).await,
        "novel bookmark detail",
    )
    .await;
    let novels = checked(
        api.get_novel_ranking_page(NovelRankingMode::Day).await,
        "current novel ranking for series",
    )
    .await;
    let novel_series_id = novels
        .novels
        .iter()
        .find_map(|n| n.series.id)
        .expect("No novel series in ranking");
    let series = checked(
        api.get_novel_series_page(novel_series_id).await,
        "novel series",
    )
    .await;
    assert_eq!(series.novel_series_detail.id, novel_series_id);
    if let Some(url) = series.next_url {
        let next = checked(
            api.get_next_novel_series_page(url).await,
            "novel series pagination",
        )
        .await;
        assert_eq!(next.novel_series_detail.id, novel_series_id);
    } else {
        println!("SKIP novel series pagination: single page");
    }
    let manga = checked(
        api.get_manga_ranking_page(MangaRankingMode::Day).await,
        "current manga ranking for series",
    )
    .await;
    let manga_series_id = manga
        .illusts
        .iter()
        .find_map(|i| i.series.as_ref().and_then(|s| s.id))
        .expect("No manga series in ranking");
    let series = checked(
        api.get_illust_series_page(manga_series_id).await,
        "manga series",
    )
    .await;
    assert_eq!(series.illust_series_detail.id, manga_series_id);
    if let Some(url) = series.next_url {
        let next = checked(
            api.get_next_illust_series_page(url).await,
            "manga series pagination",
        )
        .await;
        assert_eq!(next.illust_series_detail.id, manga_series_id);
    } else {
        println!("SKIP manga series pagination: single page");
    }
}

#[tokio::test]
#[ignore = "Uses a real Pixiv account and network; run explicitly"]
async fn live_search_filters_and_previews() {
    let api = live_api();
    let word = "初音ミク".to_string();
    let target = SearchTarget::PartialMatchForTags;

    // Popular preview: one batch of at most 30, ordered by bookmarks, no next page.
    let preview = checked(
        api.get_search_popular_preview_illust(word.clone(), target, SearchOptions::default())
            .await,
        "illustration popular preview",
    )
    .await;
    assert!(!preview.illusts.is_empty() && preview.illusts.len() <= 30);
    assert!(preview.next_url.is_none());
    assert!(
        preview
            .illusts
            .windows(2)
            .all(|pair| pair[0].total_bookmarks >= pair[1].total_bookmarks)
    );

    let novels = checked(
        api.get_search_popular_preview_novel(
            word.clone(),
            SearchNovelTarget::PartialMatchForTags,
            SearchOptions::default(),
        )
        .await,
        "novel popular preview",
    )
    .await;
    assert!(!novels.novels.is_empty() && novels.novels.len() <= 30);
    assert!(novels.next_url.is_none());

    // Novel targets that only novels have.
    for novel_target in [SearchNovelTarget::Text, SearchNovelTarget::Keyword] {
        let page = checked(
            api.get_search_novel_page(
                word.clone(),
                SearchSort::DateDesc,
                novel_target,
                SearchOptions::default(),
            )
            .await,
            "novel target",
        )
        .await;
        assert!(!page.novels.is_empty());
    }

    // Illustration filters: every result satisfies the filter it was asked for.
    let manga = checked(
        api.get_search_illust_page(
            word.clone(),
            SearchSort::DateDesc,
            target,
            SearchOptions {
                content_type: Some(SearchContentType::Manga),
                ..Default::default()
            },
        )
        .await,
        "manga content type",
    )
    .await;
    assert!(!manga.illusts.is_empty() && manga.illusts.iter().all(|i| i.kind == "manga"));

    let portrait = checked(
        api.get_search_illust_page(
            word.clone(),
            SearchSort::DateDesc,
            target,
            SearchOptions {
                ratio: Some(SearchRatio::Portrait),
                ..Default::default()
            },
        )
        .await,
        "portrait ratio",
    )
    .await;
    assert!(!portrait.illusts.is_empty() && portrait.illusts.iter().all(|i| i.height > i.width));

    let large = checked(
        api.get_search_illust_page(
            word.clone(),
            SearchSort::DateDesc,
            target,
            SearchOptions {
                width_min: Some(2000),
                ..Default::default()
            },
        )
        .await,
        "minimum width",
    )
    .await;
    assert!(!large.illusts.is_empty() && large.illusts.iter().all(|i| i.width >= 2000));

    let tooled = checked(
        api.get_search_illust_page(
            word,
            SearchSort::DateDesc,
            target,
            SearchOptions {
                tool: Some("Photoshop".into()),
                ..Default::default()
            },
        )
        .await,
        "tool",
    )
    .await;
    assert!(
        !tooled.illusts.is_empty()
            && tooled
                .illusts
                .iter()
                .all(|i| i.tools.iter().any(|tool| tool == "Photoshop"))
    );
}
