mod client;

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use anyhow::{bail, Context, Result};
use clap::Parser;
use futures::future::join_all;

use crate::client::{CandfansClient, GetUserSuccessData};

const POSTS_PER_TIMELINE: usize = 20;

#[derive(Parser)]
pub struct Cli {
    /// 対象のユーザーの ID
    target: String,

    /// ブラウザから拾ってきた Cookie ヘッダー
    #[clap(short, long)]
    cookie: String,

    /// ブラウザから拾ってきた X-XSRF-TOKEN ヘッダー
    #[clap(short, long, name = "xsrf")]
    x_xsrf_token: String,

    /// 取得開始するページ
    #[clap(short, long, default_value_t = 0)]
    offset: usize,

    /// 取得するページ数
    #[clap(short, long)]
    pages: Option<usize>,

    /// 対象の拡張子
    #[clap(short, long, default_value = "mp4")]
    extensions: Vec<String>,

    /// 出力ディレクトリ なければ target を使う
    #[clap(long)]
    output: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let Cli {
        target,
        cookie,
        x_xsrf_token,
        pages,
        offset,
        extensions,
        output,
    } = Cli::parse();

    let output = output.unwrap_or_else(|| PathBuf::from(target.clone()));

    let client = CandfansClient::new(cookie, x_xsrf_token);

    let GetUserSuccessData { user, .. } = client.get_user(&target).await?;

    let max_pages = (user.post_cnt / POSTS_PER_TIMELINE) + 1;
    let pages = pages.map(|p| p.min(max_pages)).unwrap_or(max_pages);

    let mut all_posts = Vec::with_capacity(user.post_cnt);

    for i in 0..pages {
        let page_idx = i + offset;
        let posts = client.get_post(user.id, page_idx).await?;
        all_posts.extend(posts);
    }

    log::info!("Posts = {}", all_posts.len());

    let all_paths = all_posts
        .iter()
        .flat_map(|post| post.paths())
        .collect::<Vec<_>>();

    log::info!("Paths: {}", all_paths.len());

    std::fs::create_dir_all(&output)?;

    let extensions = extensions.into_iter().collect::<HashSet<_>>();
    let count = Arc::new(AtomicUsize::new(0));
    let all = all_paths.len();
    let _ =
        join_all(all_paths.into_iter().map(|path| {
            save_content_with_log(path, &output, &extensions, Arc::clone(&count), all)
        }))
        .await;

    Ok(())
}

async fn save_content_with_log(
    path: &str,
    directory: &Path,
    exntentions: &HashSet<String>,
    count: Arc<AtomicUsize>,
    all: usize,
) {
    let count = count.fetch_add(1, Ordering::Relaxed);
    match save_content(path, directory, exntentions).await {
        Ok(true) => log::info!("Content saved ({count}/{all}): {path}"),
        Ok(false) => log::info!("Content skipped ({count}/{all}): {path}"),
        Err(e) => log::error!("Error ({count}/{all}): {path} {e:?}"),
    }
}

async fn save_content(path: &str, directory: &Path, exntentions: &HashSet<String>) -> Result<bool> {
    const HOST: &str = "https://video.candfans.jp";

    let splited = path.split("/");
    let Some(name) = splited.last() else {
        bail!("name not found");
    };

    let Some(ext) = name.split(".").last() else {
        bail!("extension not found.");
    };

    // skip
    if !exntentions.contains(ext) {
        return Ok(false);
    }

    let bytes = reqwest::get(format!("{HOST}{path}"))
        .await
        .context("request content")?
        .bytes()
        .await
        .context("get bytes")?;

    tokio::fs::write(directory.join(name), bytes)
        .await
        .context("write content")?;

    Ok(true)
}
