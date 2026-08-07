use super::{audio, video};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn audio_recommend_fetches_default_plugins_and_resolves_play_url() {
    let result = audio::recommend().await;
    println!("audio recommend items: {}", result.len());
    for item in result.iter().take(10) {
        println!(
            "audio item: category={}, name={:?}, source={}",
            item.category, item.name, item.source
        );
    }

    let item = result
        .first()
        .expect("audio recommendations should not be empty");
    let source = item.play(item.source.as_str());
    println!("audio play source: {}", source);
    assert!(source.starts_with("http"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn video_search_fetches_default_platforms_and_details() {
    let result = video::search("凡人修仙传".to_string()).await;
    println!("search platforms: {}", result.len());
    for (platform, items) in result {
        println!("search extractor={platform}, items={}", items.len());
        for item in items.iter().take(3) {
            println!("search item: name={:?}, source={}", item.name, item.source);
        }
        if let Some(item) = items.first() {
            let details = item.func.detail(item);
            println!(
                "search detail extractor={platform}, items={}",
                details.len()
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn video_recommend_fetches_default_platforms_and_details() {
    let result = video::recommend().await;
    println!("recommend items: {}", result.len());
    for item in result.iter().take(10) {
        println!(
            "recommend item: name={:?}, source={} category={}",
            item.name, item.source, item.category
        );
    }
    for item in result.iter().take(3) {
        let details = item.func.detail(item);
        println!(
            "recommend detail source={}, items={}",
            item.source,
            details.len()
        );
    }
}
