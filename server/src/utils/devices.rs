use crate::model::visit_record::ShortVisitRecord;
pub fn get_device_type(user_agent: &str) -> String {
    let user_agent = user_agent.to_lowercase();
    if user_agent.contains("iphone") {
        return "iphone".to_string();
    }
    if user_agent.contains("ipad") {
        return "iPad".to_string();
    }
    if user_agent.contains("ipod") {
        return "iPod".to_string();
    }
    if user_agent.contains("android") {
        return "Android".to_string();
    }
    if user_agent.contains("mobile") {
        return "Mobile".to_string();
    }

    if user_agent.contains("tablet") {
        return "Tablet".to_string();
    }

    "Unknown".to_string()
}


pub fn get_sys_type(user_agent: &str)->String{
    let user_agent = user_agent.to_lowercase();
    if user_agent.contains("windows") {
        return "Windows".to_string();
    }
    if user_agent.contains("macintosh") || user_agent.contains("mac os") {
        return "Mac".to_string();
    }
    if user_agent.contains("linux") {
        return "Linux".to_string();
    }

    "Unknown".to_string()
}


pub fn get_browser_type(user_agent: &str) -> String {
    let user_agent = user_agent.to_lowercase();
    if user_agent.contains("chrome") && !user_agent.contains("edg") {
        return "Google Chrome".to_string();
    }
    if user_agent.contains("firefox") {
        return "Firefox".to_string();
    }
    if user_agent.contains("safari") && !user_agent.contains("chrome") {
        return "Safari".to_string();
    }
    if user_agent.contains("edg") {
        return "Microsoft Edge".to_string();
    }
    if user_agent.contains("opera") || user_agent.contains("opr") {
        return "Opera".to_string();
    }
    if user_agent.contains("msie") || user_agent.contains("trident") {
        return "Internet Explorer".to_string();
    }

    "Unknown".to_string()

}
pub async fn get_web_info(user_agent: &str)->Option<ShortVisitRecord>{
    let device_type_info = get_device_type(user_agent);
    let sys_type_info = get_sys_type(user_agent);
    let browser_type_info = get_browser_type(user_agent);
    let visit_record = ShortVisitRecord {
        id: 0,
        short_link_id: 0,
        device_type: device_type_info,
        sys_type: sys_type_info,
        browser_type: browser_type_info,
        addr: "".to_string(),
        created_at: None,
        updated_at: None,
        deleted_at: None,
    };
    Some(visit_record)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_detection() {
        assert_eq!(get_browser_type("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"), "Google Chrome");
        assert_eq!(get_browser_type("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0"), "Firefox");
        assert_eq!(get_browser_type("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15"), "Safari");
        assert_eq!(get_browser_type("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Edg/91.0.864.59 Safari/537.36"), "Microsoft Edge");
        assert_eq!(get_browser_type("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) OPR/77.0.4054.172 Safari/537.36"), "Opera");
        assert_eq!(get_browser_type("Mozilla/5.0 (compatible; MSIE 10.0; Windows NT 6.1; Trident/6.0)"), "Internet Explorer");
        assert_eq!(get_browser_type("Unknown User Agent"), "Unknown");
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_detection() {
        assert_eq!(get_device_type("Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)"), "iPhone");
        assert_eq!(get_device_type("Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X)"), "iPad");
        assert_eq!(get_device_type("Mozilla/5.0 (iPod; CPU OS 14_0 like Mac OS X)"), "iPod");
        assert_eq!(get_device_type("Mozilla/5.0 (Linux; Android 10)"), "Android");
        assert_eq!(get_device_type("Mozilla/5.0 (Windows NT 10.0)"), "Windows");
        assert_eq!(get_device_type("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15)"), "Mac");
        assert_eq!(get_device_type("Mozilla/5.0 (X11; Linux x86_64)"), "Linux");
        assert_eq!(get_device_type("Unknown User Agent"), "Unknown");
    }
}