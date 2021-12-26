pub struct YoutubeService {
    api_keys: Vec<String>,
}

impl YoutubeService {
    pub fn new() -> YoutubeService {
        YoutubeService { api_keys: vec![] }
    }
}
