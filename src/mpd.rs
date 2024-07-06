use debug_print::debug_println;

#[derive(Default)]
struct Segment {
    t: Option<u64>,
    n: Option<u64>,
    d: u64,
    r: u64,
}

#[derive(Default)]
struct SegmentTimeline {
    segments: Vec<Segment>,
}

#[derive(Default)]
struct SegmentTemplate {
    media: Option<String>,
    initialization: Option<String>,
    segment_timeline: Option<SegmentTimeline>,
    start_number: u64,
    duration: Option<u64>,
    timescale: u64,
}

#[derive(Default)]
struct Representation {
    id: String,
    bandwidth: u64,
    segment_template: Option<SegmentTemplate>,
}

#[derive(Default)]
struct AdaptationSet {
    mime_type: String,
    segment_template: Option<SegmentTemplate>,
    representations: Vec<Representation>,
}

#[derive(Default)]
struct Period {
    adaptation_sets: Vec<AdaptationSet>,
    duration: Option<f32>,
}

#[derive(Default)]
struct MpegDash {
    periods: Vec<Period>,
    url: String,
    media_presentation_duration: Option<f32>,
}

impl MpegDash {
    fn insert(&mut self, period: Period) {
        self.periods.push(period);
    }
}

fn get_optional_attibute_from_node(node: &roxmltree::Node, attribute: &str) -> Option<String> {
    match node.attribute(attribute) {
        Some(val) => Some(val.to_string()),
        None => None,
    }
}

fn get_optional_u64_attibute_from_node(node: &roxmltree::Node, attribute: &str) -> Option<u64> {
    match node.attribute(attribute) {
        Some(val) => match val.parse() {
            Ok(num) => Some(num),
            Err(_) => None,
        },
        None => None,
    }
}

fn parse_segment_timeline_segment(node: roxmltree::Node) -> Segment {
    let mut segment = Segment {
        ..Default::default()
    };
    segment.d = match get_optional_u64_attibute_from_node(&node, "d") {
        Some(val) => val,
        None => todo!(),
    };
    segment.n = get_optional_u64_attibute_from_node(&node, "n");
    segment.r = match get_optional_u64_attibute_from_node(&node, "r") {
        Some(val) => val,
        None => 0,
    };
    segment.t = get_optional_u64_attibute_from_node(&node, "t");
    return segment;
}

fn parse_segment_timeline(node: roxmltree::Node) -> SegmentTimeline {
    let mut segment_timeline = SegmentTimeline {
        ..Default::default()
    };
    for child in node.descendants() {
        if child.has_tag_name("S") {
            segment_timeline
                .segments
                .push(parse_segment_timeline_segment(child));
        }
    }
    return segment_timeline;
}

fn parse_segment_template(node: roxmltree::Node) -> SegmentTemplate {
    let mut segment_template = SegmentTemplate {
        ..Default::default()
    };
    segment_template.initialization = get_optional_attibute_from_node(&node, "initialization");
    segment_template.media = get_optional_attibute_from_node(&node, "media");

    for child in node.descendants() {
        if child.has_tag_name("SegmentTimeline") {
            segment_template.segment_timeline = Some(parse_segment_timeline(child));
            break;
        }
    }

    match node.attribute("startNumber") {
        Some(val) => match val.parse() {
            Ok(start_number) => segment_template.start_number = start_number,
            Err(_) => segment_template.start_number = 1,
        },
        None => segment_template.start_number = 1,
    }

    match node.attribute("duration") {
        Some(val) => match val.parse() {
            Ok(duration) => segment_template.duration = Some(duration),
            Err(_) => segment_template.duration = None,
        },
        None => segment_template.duration = None,
    }

    match node.attribute("timescale") {
        Some(val) => match val.parse() {
            Ok(timescale) => segment_template.timescale = timescale,
            Err(_) => segment_template.timescale = 1,
        },
        None => segment_template.timescale = 1,
    }
    return segment_template;
}

fn check_and_parse_segment_template(node: roxmltree::Node) -> Option<SegmentTemplate> {
    let mut segment_template = Some(SegmentTemplate {
        ..Default::default()
    });
    if node.has_tag_name("SegmentTemplate") {
        segment_template = Some(parse_segment_template(node));
    }
    return segment_template;
}

fn parse_representation(node: roxmltree::Node) -> Representation {
    let mut representation = Representation {
        ..Default::default()
    };

    match node.attribute("id") {
        Some(val) => representation.id = val.to_string(),
        None => {
            eprintln!("Could not find id of representation")
        }
    }

    match node.attribute("bandwidth") {
        Some(bandwidth) => match bandwidth.parse() {
            Ok(val) => representation.bandwidth = val,
            Err(_) => eprintln!("Could not parse bandwidth of representation"),
        },
        None => {
            eprintln!("Could not find bandwidth of representation")
        }
    }

    for child in node.descendants() {
        if child.has_tag_name("SegmentTemplate") {
            representation.segment_template = check_and_parse_segment_template(child);
            break;
        }
    }
    return representation;
}

fn parse_adaptation_set(node: roxmltree::Node) -> AdaptationSet {
    let mut adaptation_set = AdaptationSet {
        ..Default::default()
    };
    match node.attribute("mimeType") {
        Some(val) => adaptation_set.mime_type = val.to_string(),
        None => {
            eprintln!("Could not find mimeType of adaptation set")
        }
    }
    for child in node.descendants() {
        if child.has_tag_name("Representation") {
            let representation = parse_representation(child);
            adaptation_set.representations.push(representation);
        } else if child.has_tag_name("SegmentTemplate") {
            adaptation_set.segment_template = Some(parse_segment_template(child));
        }
    }
    return adaptation_set;
}

fn parse_period(node: roxmltree::Node) -> Period {
    let mut period = Period {
        ..Default::default()
    };
    for child in node.descendants() {
        if child.has_tag_name("AdaptationSet") {
            let adaptation_set = parse_adaptation_set(child);
            period.adaptation_sets.push(adaptation_set);
        }
    }
    match get_optional_attibute_from_node(&node, "duration") {
        Some(duration) => {
            period.duration = duration
                .parse::<iso8601_duration::Duration>()
                .unwrap()
                .num_seconds();
        }
        None => {
            debug_println!("duration not available in period");
        }
    }
    return period;
}

fn parse_mpd(xml: String, url: String) -> MpegDash {
    let mut mpeg_dash = MpegDash {
        ..Default::default()
    };
    let result = roxmltree::Document::parse(&xml);
    match result {
        Ok(doc) => {
            for node in doc.descendants() {
                if node.is_element() {
                    if node.has_tag_name("Period") {
                        let period = parse_period(node);
                        mpeg_dash.insert(period);
                    } else if node.has_tag_name("MPD") {
                        match get_optional_attibute_from_node(&node, "mediaPresentationDuration") {
                            Some(duration) => {
                                mpeg_dash.media_presentation_duration = duration
                                    .parse::<iso8601_duration::Duration>()
                                    .unwrap()
                                    .num_seconds();
                            }
                            None => {
                                debug_println!("mediaPresentationDuration not available in MPD");
                            }
                        }
                    }
                }
            }
            mpeg_dash.url = url;
        }
        Err(e) => eprintln!("XML Parse Error: {}", e),
    }
    return mpeg_dash;
}

struct FragementDescriptor<'a> {
    number: u64,
    representation: &'a Representation,
    time: u64,
    repeat: u64,
}

fn replace_with_printf_format(template: String, identifier: &str, value: u64) -> String {
    let mut ret: String = template.clone();
    let mut token_started = false;
    let mut token: String = Default::default();
    for c in template.chars() {
        if c == '$' {
            if token_started {
                if token.starts_with(identifier) {
                    let updated: String;
                    match token.len() == identifier.len() {
                        true => {
                            updated = sprintf::sprintf!("%llu", value).unwrap();
                        }
                        false => {
                            let fmt = token.strip_prefix(identifier).unwrap();
                            updated = sprintf::sprintf!(fmt, value).unwrap();
                        }
                    }
                    let mut replacement: String = "$".to_owned();
                    replacement.push_str(&token);
                    replacement.push('$');
                    ret = ret.replace(&replacement, &updated);
                }
                token_started = false;
                token.clear();
            } else {
                token_started = true;
            }
        } else {
            if token_started {
                token.push(c);
            }
        }
    }
    return ret;
}

fn expand_segment_template(
    template_string: &str,
    fragement_descriptor: &FragementDescriptor,
) -> String {
    let mut ret: String;
    ret = template_string.to_string();
    ret = ret.replace(
        "$RepresentationID$",
        &fragement_descriptor.representation.id,
    );
    ret = replace_with_printf_format(ret, "Number", fragement_descriptor.number);
    ret = replace_with_printf_format(
        ret,
        "Bandwidth",
        fragement_descriptor.representation.bandwidth,
    );
    ret = replace_with_printf_format(ret, "Time", fragement_descriptor.time);
    return ret;
}

#[derive(Default)]
pub struct UrlInfo {
    pub base_url: String,
    pub urls: Vec<String>,
}

fn get_urls(mpd: MpegDash) -> Option<UrlInfo> {
    let mut ret: UrlInfo = UrlInfo {
        ..Default::default()
    };
    let base_url: String;
    let pos = mpd.url.rfind('/')?;
    base_url = mpd.url[..pos + 1].to_string();

    let periods_iter = mpd.periods.iter();
    for (period_idx, period) in periods_iter.enumerate() {
        debug_println!("period_idx {} ", period_idx);
        let adaptation_set_iter: std::slice::Iter<AdaptationSet> = period.adaptation_sets.iter();
        for (adaptation_set_idx, adaptation_set) in adaptation_set_iter.enumerate() {
            debug_println!(
                "adaptation_set_idx {} mimeType {}",
                adaptation_set_idx,
                adaptation_set.mime_type
            );
            let representation_iter = adaptation_set.representations.iter();
            for (representation_idx, representation) in representation_iter.enumerate() {
                debug_println!(
                    "representation_idx {} id {} bandwidth {}",
                    representation_idx,
                    representation.id,
                    representation.bandwidth
                );
                let segment_template_opt: Option<&SegmentTemplate>;
                match &representation.segment_template {
                    Some(st) => {
                        segment_template_opt = Some(st);
                    }
                    None => match &adaptation_set.segment_template {
                        Some(st) => {
                            segment_template_opt = Some(st);
                        }
                        None => {
                            segment_template_opt = None;
                        }
                    },
                }
                match segment_template_opt {
                    Some(segment_template) => {
                        let mut fragment_descriptor = FragementDescriptor {
                            number: segment_template.start_number,
                            representation,
                            time: 0,
                            repeat: 0,
                        };
                        match &segment_template.initialization {
                            Some(initialization) => {
                                let mut initialization_url = base_url.clone();
                                initialization_url.push_str(&expand_segment_template(
                                    &initialization,
                                    &fragment_descriptor,
                                ));
                                ret.urls.push(initialization_url);
                            }
                            None => {
                                eprintln!(
                                    "initialization segment is not present for rep {}",
                                    representation.id
                                )
                            }
                        }
                        match &segment_template.media {
                            Some(media) => match &segment_template.segment_timeline {
                                Some(segment_timeline) => {
                                    for s in &segment_timeline.segments {
                                        if let Some(time) = s.t {
                                            fragment_descriptor.time = time;
                                        }
                                        fragment_descriptor.repeat = s.r;
                                        loop {
                                            let mut segment_url = base_url.clone();
                                            segment_url.push_str(&expand_segment_template(
                                                media,
                                                &fragment_descriptor,
                                            ));
                                            ret.urls.push(segment_url);
                                            fragment_descriptor.time += s.d;
                                            fragment_descriptor.number += 1;
                                            if fragment_descriptor.repeat == 0 {
                                                break;
                                            }
                                            fragment_descriptor.repeat -= 1;
                                        }
                                    }
                                }
                                None => {
                                    eprintln!("Segment timeline not present");
                                    let mut total_duration: Option<f32> = period.duration;
                                    if total_duration.is_none() {
                                        total_duration = mpd.media_presentation_duration;
                                    }
                                    loop {
                                        let mut segment_url = base_url.clone();
                                        segment_url.push_str(&expand_segment_template(
                                            media,
                                            &fragment_descriptor,
                                        ));
                                        ret.urls.push(segment_url);
                                        fragment_descriptor.number += 1;

                                        match total_duration {
                                            Some(max_time) => match segment_template.duration {
                                                Some(segment_duration) => {
                                                    fragment_descriptor.time += segment_duration;
                                                    let time: f32 = (fragment_descriptor.time
                                                        / segment_template.timescale)
                                                        as f32;
                                                    if time >= max_time {
                                                        debug_println!("fragment descriptor time reached max time, break");
                                                        break;
                                                    }
                                                }
                                                None => {}
                                            },
                                            None => {
                                                eprintln!("total_duration not available");
                                                break;
                                            }
                                        }
                                    }
                                }
                            },
                            None => {
                                eprintln!("media is not present for rep {}", representation.id)
                            }
                        }
                    }
                    None => {
                        eprintln!("Segment Template not present, other formats not supported yet")
                    }
                }
            }
        }
    }
    ret.base_url = base_url;
    return Some(ret);
}

pub fn get_fragment_urls(xml_text: String, url: &str) -> Option<UrlInfo> {
    let mpd = parse_mpd(xml_text, url.to_owned());
    return get_urls(mpd);
}

#[cfg(test)]
mod tests {
    use crate::mpd::get_fragment_urls;

    use crate::mpd::expand_segment_template;
    use crate::mpd::FragementDescriptor;

    use super::Representation;

    #[test]
    fn expand_segment_template_test_1() {
        let mut representation: Representation = Default::default();
        representation.id = "repId".to_owned();
        representation.bandwidth = 12345;
        let mut template_string = "$RepresentationID$/$Number%06d$.m4s";
        let fragement_descriptor = FragementDescriptor {
            number: 1,
            representation: &representation,
            time: 123,
            repeat: 0,
        };
        assert_eq!(
            expand_segment_template(template_string, &fragement_descriptor),
            "repId/000001.m4s"
        );
        template_string = "$RepresentationID$/$Time%05d$.m4s";
        assert_eq!(
            expand_segment_template(template_string, &fragement_descriptor),
            "repId/00123.m4s"
        );
        template_string = "$RepresentationID$/$Bandwidth%07d$.m4s";
        assert_eq!(
            expand_segment_template(template_string, &fragement_descriptor),
            "repId/0012345.m4s"
        );
        template_string = "$RepresentationID$/$Bandwidth%07d$$Time%05d$$Number%06d$.m4s";
        assert_eq!(
            expand_segment_template(template_string, &fragement_descriptor),
            "repId/001234500123000001.m4s"
        );
    }

    #[test]
    fn segment_template_timeline_1() {
        let xml_text = r#"<?xml version="1.0" encoding="UTF-8"?>
                            <MPD>
                            <Period id="id_PT1S">
                            <AdaptationSet id="1">
                                <SegmentTemplate presentationTimeOffset="10399888" timescale="10000000" initialization="$RepresentationID$_$Bandwidth$_t10399888_init.mp4" media="$RepresentationID$_$Bandwidth$_t$Time$.mp4">
                                <SegmentTimeline>
                                    <S d="20480000" t="10399888"/>
                                    <S d="20480000"/>
                                </SegmentTimeline>
                                </SegmentTemplate>
                                <Representation id="audio103_3" bandwidth="460000" codecs="mp4a.40.2" audioSamplingRate="48000"></Representation>
                            </AdaptationSet>
                            </Period>
                            </MPD>"#.to_owned();
        let url_info_opt = get_fragment_urls(xml_text, "http://test.com/manifest.mpd");
        assert!(url_info_opt.is_some());
        if let Some(url_info) = url_info_opt {
            assert_eq!(url_info.urls.len(), 3);
            for url in url_info.urls.iter() {
                println!("url : {}", url);
            }
            assert!(url_info
                .urls
                .iter()
                .any(|url| url == "http://test.com/audio103_3_460000_t10399888_init.mp4"));
            assert!(url_info
                .urls
                .iter()
                .any(|url| url == "http://test.com/audio103_3_460000_t10399888.mp4"));
            assert!(url_info
                .urls
                .iter()
                .any(|url| url == "http://test.com/audio103_3_460000_t30879888.mp4"));
        }
    }

    #[test]
    fn segment_template_no_timeline_1() {
        let xml_text = r#"<?xml version="1.0"?>
        <MPD xmlns="urn:mpeg:dash:schema:mpd:2011" type="static" mediaPresentationDuration="PT0H0M60.000S">
         <Period duration="PT0H0M60.000S">
          <AdaptationSet>
           <Representation id="1" mimeType="video/mp4" bandwidth="5678742">
            <SegmentTemplate timescale="60000" media="video_8000k_$Number$.mp4" startNumber="1" duration="120000" initialization="video_8000k_init.mp4"/>
           </Representation>
          </AdaptationSet>
         </Period>
        </MPD>"#.to_owned();
        let url_info_opt = get_fragment_urls(xml_text, "http://test.com/");
        assert!(url_info_opt.is_some());
        if let Some(url_info) = url_info_opt {
            assert_eq!(url_info.urls.len(), 31);
            for url in url_info.urls.iter() {
                println!("url : {}", url);
            }
            assert!(url_info
                .urls
                .iter()
                .any(|url| url == "http://test.com/video_8000k_init.mp4"));
            assert!(url_info
                .urls
                .iter()
                .any(|url| url == "http://test.com/video_8000k_1.mp4"));
            assert!(url_info
                .urls
                .iter()
                .any(|url| url == "http://test.com/video_8000k_30.mp4"));
        }
    }
}
