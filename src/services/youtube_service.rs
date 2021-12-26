use anyhow::Error;

pub struct YoutubeService {
    api_keys: Vec<String>,
}

impl YoutubeService {
    pub fn new() -> YoutubeService {
        YoutubeService { api_keys: vec![] }
    }

    pub async fn get_statistics() -> Result<(), Error> {
        Ok(())
    }
}
