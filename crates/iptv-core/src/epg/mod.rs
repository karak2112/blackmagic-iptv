use std::collections::HashMap;

use chrono::Utc;

use crate::models::{Channel, NowNext, Programme};

#[derive(Debug, Clone, Default)]
pub struct EpgIndex {
    programmes_by_channel: HashMap<String, Vec<Programme>>,
}

impl EpgIndex {
    pub fn from_programmes(programmes: Vec<Programme>) -> Self {
        let mut programmes_by_channel: HashMap<String, Vec<Programme>> = HashMap::new();
        for prog in programmes {
            programmes_by_channel
                .entry(prog.channel_epg_id.clone())
                .or_default()
                .push(prog);
        }
        for progs in programmes_by_channel.values_mut() {
            progs.sort_by_key(|p| p.start);
        }
        Self {
            programmes_by_channel,
        }
    }

    pub fn merge(&mut self, other: EpgIndex) {
        for (channel_id, mut progs) in other.programmes_by_channel {
            let entry = self.programmes_by_channel.entry(channel_id).or_default();
            entry.append(&mut progs);
            entry.sort_by_key(|p| p.start);
            entry.dedup_by(|a, b| a.start == b.start && a.title == b.title);
        }
    }

    pub fn now_next(&self, epg_channel_id: &str) -> NowNext {
        let now = Utc::now();
        let Some(programmes) = self.programmes_by_channel.get(epg_channel_id) else {
            return NowNext::default();
        };

        let mut current = None;
        let mut upcoming = None;

        for prog in programmes {
            if prog.start <= now && prog.stop > now {
                current = Some(prog.clone());
            } else if prog.start > now {
                upcoming = Some(prog.clone());
                break;
            }
        }

        NowNext {
            now: current,
            next: upcoming,
        }
    }

    pub fn channel_count(&self) -> usize {
        self.programmes_by_channel.len()
    }

    pub fn programme_count(&self) -> usize {
        self.programmes_by_channel.values().map(|v| v.len()).sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchMethod {
    TvgId,
    TvgName,
    FuzzyName,
}

impl MatchMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TvgId => "tvg_id",
            Self::TvgName => "tvg_name",
            Self::FuzzyName => "fuzzy_name",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EpgMatch {
    pub channel_id: String,
    pub epg_channel_id: String,
    pub method: MatchMethod,
}

pub struct EpgMatcher {
    enable_fuzzy: bool,
}

impl Default for EpgMatcher {
    fn default() -> Self {
        Self {
            enable_fuzzy: false,
        }
    }
}

impl EpgMatcher {
    pub fn new(enable_fuzzy: bool) -> Self {
        Self { enable_fuzzy }
    }

    pub fn match_channels(
        &self,
        channels: &[Channel],
        epg: &EpgIndex,
    ) -> Vec<EpgMatch> {
        let epg_ids: HashMap<String, &str> = epg
            .programmes_by_channel
            .keys()
            .map(|id| (normalize_key(id), id.as_str()))
            .collect();

        let mut matches = Vec::new();

        for channel in channels {
            if let Some(tvg_id) = channel.tvg_id.as_deref().filter(|s| !s.is_empty()) {
                if epg.programmes_by_channel.contains_key(tvg_id) {
                    matches.push(EpgMatch {
                        channel_id: channel.id.clone(),
                        epg_channel_id: tvg_id.to_string(),
                        method: MatchMethod::TvgId,
                    });
                    continue;
                }
                if let Some(&epg_id) = epg_ids.get(&normalize_key(tvg_id)) {
                    matches.push(EpgMatch {
                        channel_id: channel.id.clone(),
                        epg_channel_id: epg_id.to_string(),
                        method: MatchMethod::TvgId,
                    });
                    continue;
                }
            }

            if let Some(tvg_name) = channel.tvg_name.as_deref().filter(|s| !s.is_empty()) {
                if let Some(&epg_id) = epg_ids.get(&normalize_key(tvg_name)) {
                    matches.push(EpgMatch {
                        channel_id: channel.id.clone(),
                        epg_channel_id: epg_id.to_string(),
                        method: MatchMethod::TvgName,
                    });
                    continue;
                }
            }

            if self.enable_fuzzy {
                let key = normalize_key(&channel.name);
                if let Some(&epg_id) = epg_ids.get(&key) {
                    matches.push(EpgMatch {
                        channel_id: channel.id.clone(),
                        epg_channel_id: epg_id.to_string(),
                        method: MatchMethod::FuzzyName,
                    });
                }
            }
        }

        matches
    }
}

fn normalize_key(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::channel_id;
    use chrono::TimeZone;

    fn sample_channel(tvg_id: Option<&str>, tvg_name: Option<&str>, name: &str) -> Channel {
        Channel {
            id: channel_id("src", "http://example.com/s", tvg_id),
            source_id: "src".into(),
            name: name.into(),
            group: None,
            logo_url: None,
            stream_url: "http://example.com/s".into(),
            tvg_id: tvg_id.map(str::to_string),
            tvg_name: tvg_name.map(str::to_string),
        }
    }

    fn sample_programme(channel: &str, title: &str) -> Programme {
        Programme {
            channel_epg_id: channel.into(),
            start: Utc.with_ymd_and_hms(2025, 1, 1, 6, 0, 0).unwrap(),
            stop: Utc.with_ymd_and_hms(2025, 1, 1, 7, 0, 0).unwrap(),
            title: title.into(),
            description: None,
            category: None,
        }
    }

    #[test]
    fn matches_by_tvg_id() {
        let channels = vec![sample_channel(Some("news1.us"), None, "News")];
        let epg = EpgIndex::from_programmes(vec![sample_programme("news1.us", "Show")]);
        let matcher = EpgMatcher::default();
        let matches = matcher.match_channels(&channels, &epg);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].method, MatchMethod::TvgId);
    }
}
