use std::ops::Range;
use url::Url;

/// # Jenkins Url Factor
///
/// from jenkins url like:
///
/// - "https://example.jenkins.com/job/Some.Long-JOB_NAME/1234/"
/// - "https://example.jenkins.com/view/Some_View1/view/some.View-2/job/Some.Long-JOB_NAME/1234/"
/// - "http://example.jenkins.com/view/Some_View1/view/some.View-2/job/Some.Long-JOB_NAME/1234/"
#[derive(Debug)]
pub struct JenkinsUrlFactor {
    url: Url,

    scheme_domain: Option<Range<usize>>,
    views: Vec<Range<usize>>,
    job_name: Option<Range<usize>>,
    build_number: Option<u32>,
}

impl JenkinsUrlFactor {
    pub fn from_url(jenkins_url: &str) -> Result<Self, url::ParseError> {
        let mut views = Vec::new();
        let mut scheme_domain = None;
        let mut job_name = None;
        let mut build_number = None;

        let url = Url::parse(jenkins_url)?;
        let mut cursor = 0;

        if let Some(host) = url.host_str() {
            let end = jenkins_url.find(host).unwrap_or(0) + host.len();
            scheme_domain = Some(0..end);
            cursor = end;
        }

        if let Some(path_segments) = url.path_segments() {
            let mut iter = path_segments.into_iter();
            while let Some(segment) = iter.next() {
                cursor = jenkins_url[cursor..].find(segment).unwrap_or(0) + cursor;
                cursor += segment.len();
                if segment.eq("view") {
                    if let Some(view_seg) = iter.next() {
                        cursor = jenkins_url[cursor..].find(view_seg).unwrap_or(0) + cursor;
                        views.push(cursor..(cursor + view_seg.len()));
                    }
                } else if segment.eq("job")
                    && let Some(job_name_seg) = iter.next()
                {
                    cursor = jenkins_url[cursor..].find(job_name_seg).unwrap_or(0) + cursor;
                    job_name = Some(cursor..(cursor + job_name_seg.len()));
                    build_number = iter.next().and_then(|s| s.parse::<u32>().ok());
                }
            }
        }

        Ok(JenkinsUrlFactor {
            url,
            views,
            scheme_domain,
            job_name,
            build_number,
        })
    }

    pub fn get_scheme_domain(&self) -> Option<&str> {
        self.scheme_domain
            .as_ref()
            .map(|range| &self.url.as_str()[range.clone()])
    }

    pub fn get_views(&self) -> Vec<&str> {
        self.views
            .iter()
            .map(|range| &self.url.as_str()[range.clone()])
            .collect()
    }

    pub fn get_job_name(&self) -> Option<&str> {
        self.job_name
            .as_ref()
            .map(|range| &self.url.as_str()[range.clone()])
    }

    pub fn get_build_number(&self) -> Option<u32> {
        self.build_number
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use crate::jenkins::jenkins_url_factor::JenkinsUrlFactor;

    #[test]
    fn test_parse_params_from_url() {
        let url = "https://example.jenkins.com/job/Some.Long-JOB_NAME/1234/";
        let factors = JenkinsUrlFactor::from_url(url).unwrap();
        assert_eq!(factors.views.len(), 0);
        assert_eq!(
            factors.get_scheme_domain(),
            Some("https://example.jenkins.com")
        );
        assert_eq!(factors.get_job_name(), Some("Some.Long-JOB_NAME"));
        assert_eq!(factors.build_number, Some(1234));

        let url = "https://example.jenkins.com/view/Some_View1/view/some.View-2/job/Some.Long-JOB_NAME/1234/";
        let factors = JenkinsUrlFactor::from_url(url).unwrap();
        assert_eq!(factors.views.len(), 2);
        assert_eq!(factors.get_views(), vec!["Some_View1", "some.View-2"]);
        assert_eq!(
            factors.get_scheme_domain(),
            Some("https://example.jenkins.com")
        );
        assert_eq!(factors.get_job_name(), Some("Some.Long-JOB_NAME"));
        assert_eq!(factors.build_number, Some(1234));

        let url = "http://example.jenkins.com/view/Some_View1/view/some.View-2/job/Some.Long-JOB_NAME/1234/";
        let factors = JenkinsUrlFactor::from_url(url).unwrap();
        assert_eq!(factors.views.len(), 2);
        assert_eq!(factors.get_views(), vec!["Some_View1", "some.View-2"]);
        assert_eq!(
            factors.get_scheme_domain(),
            Some("http://example.jenkins.com")
        );
        assert_eq!(factors.get_job_name(), Some("Some.Long-JOB_NAME"));
        assert_eq!(factors.build_number, Some(1234));
    }
}
