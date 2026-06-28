#[test]
fn database_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db = super::Database::open(&db_path).unwrap();

    use chrono::Utc;
    use iptv_core::{Channel, Programme, Source, SourceType, channel_id};
    use iptv_core::epg::{EpgIndex, EpgMatcher, MatchMethod};

    let source = Source {
        id: "src1".into(),
        name: "Test".into(),
        source_type: SourceType::M3uLocal,
        path_or_url: "/tmp/test.m3u".into(),
        last_loaded: Some(Utc::now()),
    };
    db.upsert_source(&source).unwrap();

    let channels = vec![Channel {
        id: channel_id("src1", "http://example.com/s1", Some("news1.us")),
        source_id: "src1".into(),
        name: "News".into(),
        group: Some("News".into()),
        logo_url: None,
        stream_url: "http://example.com/s1".into(),
        tvg_id: Some("news1.us".into()),
        tvg_name: None,
    }];
    db.replace_channels("src1", &channels).unwrap();

    let programmes = vec![Programme {
        channel_epg_id: "news1.us".into(),
        start: Utc::now(),
        stop: Utc::now() + chrono::Duration::hours(1),
        title: "Live News".into(),
        description: None,
        category: None,
    }];
    db.replace_programmes(&programmes).unwrap();

    let epg = EpgIndex::from_programmes(programmes);
    let matches = EpgMatcher::default().match_channels(&channels, &epg);
    db.replace_epg_matches(&matches).unwrap();

    let (loaded, total) = db.list_channels(None, None, None, false, 0, 10).unwrap();
    assert_eq!(total, 1);
    assert_eq!(loaded[0].name, "News");

    let summary = db.epg_summary().unwrap();
    assert_eq!(summary.matched_count, 1);
    assert_eq!(matches[0].method, MatchMethod::TvgId);
}
