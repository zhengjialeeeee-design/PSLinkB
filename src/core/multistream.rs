//! Multi-platform RTMP output planning.
//!
//! This module builds an FFmpeg tee-muxer target list. It is intentionally
//! independent from the live state machine so it can be tested before wiring
//! platform lifecycle/API calls into the actor.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtmpTarget {
    pub platform: String,
    pub url: String,
    pub stream_key: String,
    pub enabled: bool,
}

impl RtmpTarget {
    pub fn endpoint(&self) -> Result<String, String> {
        if !self.enabled {
            return Err(format!("{} output is disabled", self.platform));
        }
        if self.url.trim().is_empty() || self.stream_key.trim().is_empty() {
            return Err(format!("{} RTMP URL or stream key is empty", self.platform));
        }
        Ok(format!("{}{}", self.url, self.stream_key))
    }
}

/// Build one FFmpeg tee-muxer URL for independent FLV/RTMP destinations.
/// The caller must pass this as a single process argument (never through a shell).
pub fn build_tee_url(targets: &[RtmpTarget]) -> Result<String, String> {
    let mut outputs = Vec::new();
    for target in targets.iter().filter(|t| t.enabled) {
        let endpoint = target.endpoint()?;
        let escaped = escape_tee_url(&endpoint);
        outputs.push(format!("[f=flv:onfail=ignore]{escaped}"));
    }
    if outputs.is_empty() {
        return Err("no enabled RTMP output targets".into());
    }
    Ok(outputs.join("|"))
}

fn escape_tee_url(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_two_independent_outputs() {
        let targets = vec![
            RtmpTarget { platform: "bilibili".into(), url: "rtmp://bili/app/".into(), stream_key: "key1".into(), enabled: true },
            RtmpTarget { platform: "douyin".into(), url: "rtmp://douyin/live/".into(), stream_key: "key2".into(), enabled: true },
        ];
        let tee = build_tee_url(&targets).unwrap();
        assert!(tee.contains("[f=flv:onfail=ignore]rtmp://bili/app/key1"));
        assert!(tee.contains("[f=flv:onfail=ignore]rtmp://douyin/live/key2"));
        assert_eq!(tee.matches("[f=flv:onfail=ignore]").count(), 2);
    }

    #[test]
    fn disabled_targets_are_skipped() {
        let targets = vec![
            RtmpTarget { platform: "bilibili".into(), url: "rtmp://bili/app/".into(), stream_key: "key1".into(), enabled: true },
            RtmpTarget { platform: "douyin".into(), url: "rtmp://douyin/live/".into(), stream_key: "key2".into(), enabled: false },
        ];
        let tee = build_ee_url(&targets).unwrap();
        assert!(tee.contains("bili"));
        assert!(!tee.contains("douyin"));
    }

    #[test]
    fn rejects_empty_output_set() {
        assert!(build_tee_url(&[]).is_err());
    }

    #[test]
    fn escapes_tee_control_characters() {
        assert_eq!(escape_tee_url("rtmp://host/a|b[c]"), "rtmp://host/a\\|b\\[c\\]");
    }
}
