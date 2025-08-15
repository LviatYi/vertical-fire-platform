/// # Jenkins Url Factor
///
/// from jenkins url like:
///
/// - "https://example.jenkins.com/job/Some.Long-JOB_NAME/1234/"
/// - "https://example.jenkins.com/view/Some_View1/view/some.View-2/job/Some.Long-JOB_NAME/1234/"
/// - "http://example.jenkins.com/view/Some_View1/view/some.View-2/job/Some.Long-JOB_NAME/1234/"
#[derive(Debug)]
pub struct JenkinsUrlFactor {
    pub views: Vec<String>,
    pub prefix_with_domain: Option<String>,
    pub job_name: Option<String>,
    pub build_number: Option<u32>,
}

impl JenkinsUrlFactor {
    pub fn from_url(jenkins_url: &str) -> Self {
        let mut views = Vec::new();
        let mut prefix_with_domain = None;
        let mut job_name = None;
        let mut build_number = None;

        if let Ok(url) = url::Url::parse(jenkins_url) {
            if let Some(host) = url.host_str() {
                prefix_with_domain = Some(format!("{}://{}", url.scheme(), host));
            }

            if let Some(path_segments) = url.path_segments() {
                let mut iter = path_segments.into_iter();

                while let Some(segment) = iter.next() {
                    if segment.eq("view") {
                        if let Some(view_seg) = iter.next() {
                            views.push(view_seg.to_string());
                        }
                    } else if segment.eq("job") {
                        job_name = iter.next().map(|s| s.to_string());
                        build_number = iter.next().and_then(|s| s.parse::<u32>().ok());
                    }
                }
            }
        }

        JenkinsUrlFactor {
            views,
            prefix_with_domain,
            job_name,
            build_number,
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use crate::jenkins::jenkins_url_factor::JenkinsUrlFactor;

    #[test]
    fn test_parse_params_from_url() {
        let url = "https://example.jenkins.com/job/Some.Long-JOB_NAME/1234/";
        let factors = JenkinsUrlFactor::from_url(url);
        assert_eq!(factors.views.len(), 0);
        assert_eq!(
            factors.prefix_with_domain,
            Some("https://example.jenkins.com".to_string())
        );
        assert_eq!(factors.job_name, Some("Some.Long-JOB_NAME".to_string()));
        assert_eq!(factors.build_number, Some(1234));

        let url = "https://example.jenkins.com/view/Some_View1/view/some.View-2/job/Some.Long-JOB_NAME/1234/";
        let factors = JenkinsUrlFactor::from_url(url);
        assert_eq!(factors.views.len(), 2);
        assert_eq!(
            factors.views,
            vec!["Some_View1".to_string(), "some.View-2".to_string()]
        );
        assert_eq!(
            factors.prefix_with_domain,
            Some("https://example.jenkins.com".to_string())
        );
        assert_eq!(factors.job_name, Some("Some.Long-JOB_NAME".to_string()));
        assert_eq!(factors.build_number, Some(1234));

        let url = "http://example.jenkins.com/view/Some_View1/view/some.View-2/job/Some.Long-JOB_NAME/1234/";
        let factors = JenkinsUrlFactor::from_url(url);
        assert_eq!(factors.views.len(), 2);
        assert_eq!(
            factors.views,
            vec!["Some_View1".to_string(), "some.View-2".to_string()]
        );
        assert_eq!(
            factors.prefix_with_domain,
            Some("http://example.jenkins.com".to_string())
        );
        assert_eq!(factors.job_name, Some("Some.Long-JOB_NAME".to_string()));
        assert_eq!(factors.build_number, Some(1234));
    }
}
