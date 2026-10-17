//! Application catalog search and ranking.
//!
//! Pure ranking over discovered applications; discovery and launching live in
//! the platform layer.

use nalios_core::apps::DesktopApp;

/// Rank application indices for a query.
///
/// Favorites come first for an empty query; for a real query they receive a
/// score bonus but must still match. An empty query otherwise falls back to
/// catalog order.
pub fn rank(apps: &[DesktopApp], favorites: &[String], query: &str, limit: usize) -> Vec<usize> {
    let query = query.trim().to_lowercase();

    if query.is_empty() {
        let mut ordered: Vec<usize> = Vec::new();
        for favorite in favorites {
            if let Some(index) = apps.iter().position(|app| app.id == *favorite) {
                if !ordered.contains(&index) {
                    ordered.push(index);
                }
            }
        }
        for index in 0..apps.len() {
            if !ordered.contains(&index) {
                ordered.push(index);
            }
        }
        ordered.truncate(limit);
        return ordered;
    }

    let mut scored: Vec<(i32, usize)> = apps
        .iter()
        .enumerate()
        .filter_map(|(index, app)| {
            score(app, &query).map(|score| {
                let favorite = favorites.iter().any(|favorite| *favorite == app.id);
                (score + if favorite { 50 } else { 0 }, index)
            })
        })
        .collect();

    scored.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| apps[a.1].name.to_lowercase().cmp(&apps[b.1].name.to_lowercase()))
    });

    scored.into_iter().take(limit).map(|(_, index)| index).collect()
}

fn score(app: &DesktopApp, query: &str) -> Option<i32> {
    let name = app.name.to_lowercase();

    if name == query {
        return Some(100);
    }
    if name.starts_with(query) {
        return Some(80);
    }
    if name
        .split_whitespace()
        .any(|word| word.starts_with(query))
    {
        return Some(60);
    }
    if name.contains(query) {
        return Some(40);
    }
    if app.id.to_lowercase().contains(query) {
        return Some(20);
    }
    if app
        .keywords
        .iter()
        .any(|keyword| keyword.to_lowercase().contains(query))
    {
        return Some(10);
    }
    if app
        .comment
        .as_deref()
        .is_some_and(|comment| comment.to_lowercase().contains(query))
    {
        return Some(5);
    }

    None
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::path::PathBuf;

    use super::*;

    fn app(id: &str, name: &str, comment: Option<&str>, keywords: &[&str]) -> DesktopApp {
        DesktopApp {
            id: id.to_owned(),
            name: name.to_owned(),
            comment: comment.map(str::to_owned),
            icon: None,
            terminal: false,
            path: PathBuf::from(format!("/tmp/{id}")),
            exec: vec!["/bin/true".to_owned()],
            keywords: keywords.iter().map(|keyword| (*keyword).to_owned()).collect(),
        }
    }

    fn fixture() -> Vec<DesktopApp> {
        vec![
            app("firefox.desktop", "Firefox", Some("Web browser"), &["browser"]),
            app("org.gnome.Terminal.desktop", "Terminal", None, &["shell"]),
            app("code.desktop", "Visual Studio Code", Some("Code editor"), &["editor"]),
            app("files.desktop", "Files", None, &[]),
        ]
    }

    #[test]
    fn empty_query_returns_catalog_order() {
        let apps = fixture();

        let ranked = rank(&apps, &[], "", 2);

        assert_eq!(ranked, vec![0, 1]);
    }

    #[test]
    fn exact_and_prefix_matches_rank_first() {
        let apps = fixture();

        let ranked = rank(&apps, &[], "fire", 4);

        assert_eq!(apps[ranked[0]].id, "firefox.desktop");
    }

    #[test]
    fn word_prefix_matches_work() {
        let apps = fixture();

        let ranked = rank(&apps, &[], "code", 4);

        assert_eq!(apps[ranked[0]].id, "code.desktop");
    }

    #[test]
    fn keywords_and_comments_are_fallbacks() {
        let apps = fixture();

        let by_keyword = rank(&apps, &[], "editor", 4);
        assert_eq!(apps[by_keyword[0]].id, "code.desktop");

        let by_comment = rank(&apps, &[], "browser", 4);
        assert_eq!(apps[by_comment[0]].id, "firefox.desktop");
    }

    #[test]
    fn unmatched_queries_return_nothing() {
        let apps = fixture();

        assert!(rank(&apps, &[], "zzzz-no-match", 4).is_empty());
    }

    #[test]
    fn limit_is_respected() {
        let apps = fixture();

        assert_eq!(rank(&apps, &[], "e", 1).len(), 1);
    }

    #[test]
    fn favorites_come_first_for_an_empty_query() {
        let apps = fixture();
        let favorites = vec!["code.desktop".to_owned(), "files.desktop".to_owned()];

        let ranked = rank(&apps, &favorites, "", 4);

        assert_eq!(ranked[0], 2);
        assert_eq!(ranked[1], 3);
        assert_eq!(ranked.len(), 4);
    }

    #[test]
    fn favorites_are_boosted_for_matching_queries() {
        let apps = fixture();
        let favorites = vec!["files.desktop".to_owned()];

        let ranked = rank(&apps, &favorites, "file", 4);

        assert_eq!(apps[ranked[0]].id, "files.desktop");
    }
}
