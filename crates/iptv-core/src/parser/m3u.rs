use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

use chrono::{DateTime, Utc};

use crate::error::{IptvError, Result};
use crate::models::{channel_id, Channel};
use crate::validation::UrlValidator;

#[derive(Debug, Clone, Default)]
pub struct M3uParseProgress {
    pub channels_parsed: usize,
    pub bytes_read: usize,
}

pub struct M3uParser {
    validator: UrlValidator,
}

impl Default for M3uParser {
    fn default() -> Self {
        Self {
            validator: UrlValidator::default(),
        }
    }
}

impl M3uParser {
    pub fn new(validator: UrlValidator) -> Self {
        Self { validator }
    }

    pub fn parse_reader<R: Read>(
        &self,
        source_id: &str,
        reader: R,
    ) -> Result<Vec<Channel>> {
        self.parse_reader_with_progress(source_id, reader, |_| {})
    }

    pub fn parse_reader_with_progress<R: Read, F: FnMut(M3uParseProgress)>(
        &self,
        source_id: &str,
        reader: R,
        mut on_progress: F,
    ) -> Result<Vec<Channel>> {
        let mut channels = Vec::new();
        let mut pending_extinf: Option<ExtInf> = None;
        let mut bytes_read = 0usize;

        for line in BufReader::new(reader).lines() {
            let line = line?;
            bytes_read += line.len() + 1;

            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("#EXTM3U") {
                continue;
            }

            if trimmed.starts_with("#EXTINF:") {
                pending_extinf = Some(parse_extinf(trimmed)?);
                continue;
            }

            if trimmed.starts_with('#') {
                continue;
            }

            if let Some(info) = pending_extinf.take() {
                let stream_url = self.validator.validate_stream_url(trimmed)?;
                let id = channel_id(source_id, &stream_url, info.tvg_id.as_deref());
                channels.push(Channel {
                    id,
                    source_id: source_id.to_string(),
                    name: info.name,
                    group: info.group_title,
                    logo_url: info.tvg_logo,
                    stream_url,
                    tvg_id: info.tvg_id,
                    tvg_name: info.tvg_name,
                });

                if channels.len().is_multiple_of(100) {
                    on_progress(M3uParseProgress {
                        channels_parsed: channels.len(),
                        bytes_read,
                    });
                }
            }
        }

        on_progress(M3uParseProgress {
            channels_parsed: channels.len(),
            bytes_read,
        });

        Ok(channels)
    }
}

#[derive(Debug, Clone)]
struct ExtInf {
    name: String,
    tvg_id: Option<String>,
    tvg_name: Option<String>,
    tvg_logo: Option<String>,
    group_title: Option<String>,
}

fn parse_extinf(line: &str) -> Result<ExtInf> {
    let attrs_and_name = line
        .strip_prefix("#EXTINF:")
        .ok_or_else(|| IptvError::Parse("invalid EXTINF".into()))?;

    let (attrs_part, name) = if let Some(comma_idx) = attrs_and_name.rfind(',') {
        (
            &attrs_and_name[..comma_idx],
            attrs_and_name[comma_idx + 1..].trim().to_string(),
        )
    } else {
        (attrs_and_name, String::from("Unknown"))
    };

    let attrs = parse_attributes(attrs_part);

    Ok(ExtInf {
        name: if name.is_empty() {
            attrs
                .get("tvg-name")
                .cloned()
                .unwrap_or_else(|| "Unknown".into())
        } else {
            name
        },
        tvg_id: attrs.get("tvg-id").filter(|s| !s.is_empty()).cloned(),
        tvg_name: attrs.get("tvg-name").cloned(),
        tvg_logo: attrs.get("tvg-logo").cloned(),
        group_title: attrs.get("group-title").cloned(),
    })
}

fn parse_attributes(input: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    let mut rest = input.trim();

    while let Some(key_start) = rest.find(|c: char| c.is_ascii_alphabetic()) {
        rest = &rest[key_start..];
        let Some(eq) = rest.find('=') else {
            break;
        };
        let key = rest[..eq].trim().to_ascii_lowercase();
        rest = &rest[eq + 1..].trim_start();

        let value = if rest.starts_with('"') {
            let end = rest[1..]
                .find('"')
                .map(|i| i + 2)
                .unwrap_or(rest.len());
            let v = rest[1..end - 1].to_string();
            rest = &rest[end..];
            v
        } else {
            let end = rest
                .find(|c: char| c.is_whitespace())
                .unwrap_or(rest.len());
            let (v, next) = rest.split_at(end);
            rest = next;
            v.to_string()
        };

        attrs.insert(key, value);
    }

    attrs
}

pub fn parse_xmltv_timestamp(raw: &str) -> Result<DateTime<Utc>> {
    let trimmed = raw.trim();
    let formats = [
        "%Y%m%d%H%M%S %z",
        "%Y%m%d%H%M%S",
        "%Y-%m-%d %H:%M:%S %z",
        "%Y-%m-%dT%H:%M:%S%.f%z",
        "%Y-%m-%dT%H:%M:%S%.fZ",
    ];

    for fmt in formats {
        if let Ok(dt) = DateTime::parse_from_str(trimmed, fmt) {
            return Ok(dt.with_timezone(&Utc));
        }
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(trimmed, fmt) {
            return Ok(naive.and_utc());
        }
    }

    Err(IptvError::Parse(format!("invalid XMLTV timestamp: {trimmed}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sample_playlist() {
        let data = include_str!("../../tests/fixtures/sample.m3u");
        let parser = M3uParser::default();
        let channels = parser.parse_reader("src1", data.as_bytes()).unwrap();
        assert_eq!(channels.len(), 3);
        assert_eq!(channels[0].name, "News One");
        assert_eq!(channels[0].tvg_id.as_deref(), Some("news1.us"));
        assert_eq!(channels[0].group.as_deref(), Some("News"));
    }

    #[test]
    fn parses_attributes() {
        let attrs = parse_attributes(r#"-1 tvg-id="a" tvg-name="B" group-title="Sports""#);
        assert_eq!(attrs.get("tvg-id").map(String::as_str), Some("a"));
        assert_eq!(attrs.get("group-title").map(String::as_str), Some("Sports"));
    }

    #[test]
    fn duplicate_tvg_id_gets_distinct_channel_ids() {
        let a = channel_id("src", "http://example.com/a", Some("espn.us"));
        let b = channel_id("src", "http://example.com/b", Some("espn.us"));
        assert_ne!(a, b);
    }
}
