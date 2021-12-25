#[tokio::main]
pub async fn main() -> Result<(), anyhow::Error> {
    loop {
        let mut crawler = Crawler::new();
        crawler.crawl().await?;

        tokio::time::delay_for(Duration::from_secs(900)).await;
    }
}
