use crate::repos::non_sailing_channel_repo::NonSailingChannelRepository;

pub struct SailingTermResult {
    pub has_sailing_term: bool,
    pub is_blacklisted: bool,
}

pub struct SailingTermsService {
    sailing_terms: Vec<String>,
    blacklisted_channel_ids: Vec<String>,
    non_sailing_channel_repo: NonSailingChannelRepository,
}

impl SailingTermsService {
    pub fn new(
        sailing_terms: Vec<String>,
        blacklisted_channel_ids: Vec<String>,
        non_sailing_channel_repo: NonSailingChannelRepository,
    ) -> SailingTermsService {
        SailingTermsService {
            sailing_terms,
            blacklisted_channel_ids,
            non_sailing_channel_repo,
        }
    }

    pub async fn is_not_listed_as_non_sailing_channel(&self, channel_id: &str) -> bool {
        let non_sailing_channel_exists = self
            .non_sailing_channel_repo
            .exists(channel_id)
            .await
            .unwrap_or(false);

        !non_sailing_channel_exists
    }

    pub async fn has_sailing_term(
        &self,
        channel_id: &str,
        channel_title: &str,
        channel_description: &str,
        ignore_sailing_terms: bool,
    ) -> SailingTermResult {
        let mut has_sailing_term = false;
        let mut is_blacklisted = false;

        for term in &self.sailing_terms {
            if channel_title.to_lowercase().contains(term)
                || channel_description.to_lowercase().contains(term)
            {
                has_sailing_term = true;
                break;
            }
        }

        if has_sailing_term == false && ignore_sailing_terms == false {
            self.non_sailing_channel_repo.upsert(&channel_id).await;
        }

        if ignore_sailing_terms == true {
            has_sailing_term = true;
        }

        if self
            .blacklisted_channel_ids
            .contains(&channel_id.to_string())
        {
            has_sailing_term = false;
            is_blacklisted = true;
            //self.delete_channel(channel_id).await.unwrap();
        }

        SailingTermResult {
            has_sailing_term,
            is_blacklisted,
        }
    }
}
