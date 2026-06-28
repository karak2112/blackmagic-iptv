use std::collections::HashMap;
use std::io::{BufReader, Read};

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::error::{IptvError, Result};
use crate::models::Programme;
use crate::parser::m3u::parse_xmltv_timestamp;

#[derive(Debug, Clone, Default)]
pub struct XmltvParseProgress {
    pub programmes_parsed: usize,
    pub channels_found: usize,
}

pub struct XmltvParser;

impl XmltvParser {
    pub fn parse_reader<R: Read>(reader: R) -> Result<Vec<Programme>> {
        Self::parse_reader_with_progress(reader, |_| {})
    }

    pub fn parse_reader_with_progress<R: Read, F: FnMut(XmltvParseProgress)>(
        reader: R,
        mut on_progress: F,
    ) -> Result<Vec<Programme>> {
        let mut xml = Reader::from_reader(BufReader::new(reader));
        xml.config_mut().trim_text(true);

        let mut programmes = Vec::new();
        let mut epg_channels: HashMap<String, String> = HashMap::new();
        let mut buf = Vec::new();

        let mut in_channel = false;
        let mut in_programme = false;
        let mut current_channel_id: Option<String> = None;
        let mut current_display_name = String::new();

        let mut prog_channel = String::new();
        let mut prog_start = String::new();
        let mut prog_stop = String::new();
        let mut prog_title = String::new();
        let mut prog_desc = String::new();
        let mut prog_category = String::new();
        let mut current_text_field: Option<&'static str> = None;

        loop {
            match xml.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = e.name().as_ref().to_vec();
                    match name.as_slice() {
                        b"channel" => {
                            in_channel = true;
                            current_channel_id = e
                                .attributes()
                                .filter_map(|a| a.ok())
                                .find(|a| a.key.as_ref() == b"id")
                                .and_then(|a| String::from_utf8(a.value.into_owned()).ok());
                            current_display_name.clear();
                        }
                        b"programme" => {
                            in_programme = true;
                            prog_channel.clear();
                            prog_start.clear();
                            prog_stop.clear();
                            prog_title.clear();
                            prog_desc.clear();
                            prog_category.clear();

                            for attr in e.attributes().flatten() {
                                match attr.key.as_ref() {
                                    b"channel" => {
                                        prog_channel =
                                            String::from_utf8(attr.value.into_owned()).unwrap_or_default();
                                    }
                                    b"start" => {
                                        prog_start =
                                            String::from_utf8(attr.value.into_owned()).unwrap_or_default();
                                    }
                                    b"stop" => {
                                        prog_stop =
                                            String::from_utf8(attr.value.into_owned()).unwrap_or_default();
                                    }
                                    _ => {}
                                }
                            }
                        }
                        b"display-name" if in_channel => current_text_field = Some("display-name"),
                        b"title" if in_programme => current_text_field = Some("title"),
                        b"desc" if in_programme => current_text_field = Some("desc"),
                        b"category" if in_programme => current_text_field = Some("category"),
                        _ => {}
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape()?.into_owned();
                    match current_text_field {
                        Some("display-name") => current_display_name = text,
                        Some("title") => prog_title = text,
                        Some("desc") => prog_desc = text,
                        Some("category") => prog_category = text,
                        _ => {}
                    }
                }
                Ok(Event::End(e)) => {
                    match e.name().as_ref() {
                        b"channel" => {
                            if let Some(id) = current_channel_id.take() {
                                if !current_display_name.is_empty() {
                                    epg_channels.insert(id.clone(), current_display_name.clone());
                                } else {
                                    epg_channels.insert(id, String::new());
                                }
                            }
                            in_channel = false;
                            on_progress(XmltvParseProgress {
                                programmes_parsed: programmes.len(),
                                channels_found: epg_channels.len(),
                            });
                        }
                        b"programme" => {
                            if !prog_channel.is_empty()
                                && !prog_start.is_empty()
                                && !prog_stop.is_empty()
                                && !prog_title.is_empty()
                            {
                                programmes.push(Programme {
                                    channel_epg_id: prog_channel.clone(),
                                    start: parse_xmltv_timestamp(&prog_start)?,
                                    stop: parse_xmltv_timestamp(&prog_stop)?,
                                    title: prog_title.clone(),
                                    description: if prog_desc.is_empty() {
                                        None
                                    } else {
                                        Some(prog_desc.clone())
                                    },
                                    category: if prog_category.is_empty() {
                                        None
                                    } else {
                                        Some(prog_category.clone())
                                    },
                                });

                                if programmes.len().is_multiple_of(500) {
                                    on_progress(XmltvParseProgress {
                                        programmes_parsed: programmes.len(),
                                        channels_found: epg_channels.len(),
                                    });
                                }
                            }
                            in_programme = false;
                        }
                        b"display-name" | b"title" | b"desc" | b"category" => {
                            current_text_field = None;
                        }
                        _ => {}
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(IptvError::Parse(format!("XML error: {e}"))),
                _ => {}
            }
            buf.clear();
        }

        programmes.sort_by(|a, b| {
            a.channel_epg_id
                .cmp(&b.channel_epg_id)
                .then(a.start.cmp(&b.start))
        });

        on_progress(XmltvParseProgress {
            programmes_parsed: programmes.len(),
            channels_found: epg_channels.len(),
        });

        Ok(programmes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sample_guide() {
        let data = include_str!("../../tests/fixtures/sample.xmltv");
        let programmes = XmltvParser::parse_reader(data.as_bytes()).unwrap();
        assert_eq!(programmes.len(), 3);
        assert_eq!(programmes[0].title, "Morning News");
        assert_eq!(programmes[0].channel_epg_id, "news1.us");
    }
}
