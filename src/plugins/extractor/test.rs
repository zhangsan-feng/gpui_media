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
    let source = item.func.play(item);
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
async fn video_recommend_to_detail_to_play_resolves_source() {
    let recommendations = video::recommend().await;
    println!("[test:video] recommend items={}", recommendations.len());
    for item in recommendations.iter().take(10) {
        println!(
            "[test:video] recommend id={} name={:?} source={} category={}",
            item.id, item.name, item.source, item.category
        );
    }

    assert!(
        !recommendations.is_empty(),
        "video recommendations should not be empty"
    );

    let mut successful_chain = None;
    for item in recommendations.iter().take(30) {
        println!(
            "[test:video] detail request id={} source={}",
            item.id, item.source
        );
        let details = item.func.detail(item);
        println!(
            "[test:video] detail response id={} episodes={}",
            item.id,
            details.len()
        );
        for episode in details.iter().take(5) {
            println!(
                "[test:video] episode id={} name={:?} source={} headers={:?}",
                episode.id,
                episode.name,
                episode.source,
                episode
                    .headers
                    .keys()
                    .map(|name| name.as_str())
                    .collect::<Vec<_>>()
            );
        }

        let Some(episode) = details.first() else {
            continue;
        };
        assert!(
            !episode.headers.is_empty(),
            "video episode must carry browser headers into playback: episode_id={}",
            episode.id
        );
        let play_source = episode.func.play(episode);
        println!(
            "[test:video] play resolved episode_id={} source={}",
            episode.id, play_source
        );
        if play_source.trim().is_empty() {
            continue;
        }

        successful_chain = Some((item.id.clone(), episode.id.clone(), play_source));
        break;
    }

    let (item_id, episode_id, play_source) = successful_chain.expect(
        "no recommendation completed the detail-to-play chain; inspect the printed request logs",
    );
    assert!(
        !play_source.trim().is_empty(),
        "video episode should resolve a non-empty play source: item_id={item_id}, episode_id={episode_id}"
    );
}
