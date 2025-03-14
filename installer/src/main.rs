use std::io::Read;
use tokio::io::AsyncWriteExt;
mod log;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use strum::IntoEnumIterator;

#[derive(
    clap::ValueEnum, Debug, Copy, Clone, Default, PartialEq, strum::Display, strum::EnumIter,
)]
enum Device {
    #[clap(name = "cpu")]
    #[strum(to_string = "CPU")]
    #[default]
    Cpu,
    #[clap(name = "cuda")]
    #[strum(to_string = "CUDA")]
    Cuda,
}

#[derive(clap::Parser, Debug)]
struct Opts {
    #[clap(short, long)]
    device: Option<Device>,
}

static VERSION: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    std::env::var("ZTS_VERSION").expect("バージョンが設定されていません")
});

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseInfo {
    compressed_size: u64,
    decompressed_size: u64,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();
    let result = main_inner(opts).await;
    if let Err(ref err) = result {
        error!("エラーが発生しました：{:#?}", err);
    }

    info!("Enterキーを押すと終了します...");
    let mut stdin = std::io::stdin();
    let _ = stdin.read(&mut [0u8]).unwrap();
    std::process::exit(result.is_err() as i32);
}

async fn main_inner(opts: Opts) -> Result<()> {
    let standard_font = figlet_rs::FIGfont::standard().unwrap();
    let figure = standard_font.convert("ZundaArrow TTS").unwrap();
    println!("{}", figure.to_string().trim_end().green());
    println!();
    println!(
        "  Developed by {} | Based on Zundamon Speech by {}",
        "Nanashi.".truecolor(0x48, 0xb0, 0xd5),
        "Tohoku Zunko / Zundamon Project".green()
    );
    println!();
    let device = opts.device.unwrap_or_else(|| {
        let items = Device::iter().collect::<Vec<_>>();
        let index = dialoguer::Select::new()
            .with_prompt("ハードウェアアクセラレーションを選択してください")
            .items(&items)
            .default(0)
            .interact()
            .unwrap();
        items[index]
    });

    info!("バージョン：{}", &VERSION.clone());
    info!("ハードウェア：{:?}", device);

    let release_assets = get_release_assets(&VERSION, device).await?;
    let release_info = release_assets
        .iter()
        .find(|asset| asset.name.ends_with(".meta.json"))
        .ok_or_else(|| anyhow::anyhow!("リリース情報が見つかりませんでした"))?;

    let release_info = reqwest::get(release_info.browser_download_url.clone())
        .await?
        .json::<ReleaseInfo>()
        .await?;

    info!("インストール先を選択してください。");
    let install_dir = rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .ok_or_else(|| anyhow::anyhow!("インストール先が選択されませんでした"))?;
    let install_dir = install_dir.path();

    info!(
        "ダウンロードするファイルの容量：{}",
        indicatif::HumanBytes(release_info.compressed_size)
    );
    info!(
        "解凍時のディスク使用量：{}",
        indicatif::HumanBytes(release_info.decompressed_size)
    );
    info!("インストール先：{}", install_dir.display());

    if !dialoguer::Confirm::new()
        .with_prompt("実行しますか？")
        .interact()?
    {
        return Ok(());
    }

    let mut szr = reqwest::get("https://www.7-zip.org/a/7zr.exe").await?;
    let szr_path = install_dir.join("7zr.exe");
    let mut szr_file = tokio::fs::File::create(&szr_path).await?;
    while let Some(chunk) = szr.chunk().await? {
        szr_file.write_all(&chunk).await?;
    }

    let download_map = release_assets
        .iter()
        .filter_map(|asset| {
            if asset.name.ends_with(".meta.json") {
                return None;
            }
            let url = asset.browser_download_url.clone();
            let dest = install_dir.join(asset.name.clone());
            Some((url, dest))
        })
        .collect::<Vec<_>>();
    download_urls(download_map).await?;

    info!("解凍中...");

    let mut command = tokio::process::Command::new(&szr_path);
    command.arg("x");
    command.arg("-y");
    command.arg("-o").arg(install_dir);
    for asset in &release_assets {
        command.arg(install_dir.join(asset.name.clone()));
    }
    command.stdout(std::process::Stdio::inherit());
    command.stderr(std::process::Stdio::inherit());

    let status = command.status().await?;

    if !status.success() {
        return Err(anyhow::anyhow!("7zrの実行に失敗しました"));
    }

    tokio::fs::remove_file(szr_path).await?;
    for asset in release_assets {
        tokio::fs::remove_file(install_dir.join(asset.name)).await?;
    }

    info!("インストールが完了しました。");

    Ok(())
}

async fn get_release_assets(
    version: &str,
    device: Device,
) -> Result<Vec<octocrab::models::repos::Asset>> {
    info!("ファイル一覧を取得中...");
    let assets = octocrab::instance()
        .repos("sevenc-nanashi", "zundaarrow-tts")
        .releases()
        .get_by_tag(version)
        .await?
        .assets;
    Ok(assets
        .into_iter()
        .filter(|asset| {
            asset.name.contains("windows")
                && asset.name.contains(&device.to_string().to_lowercase())
        })
        .collect())
}

async fn download_urls(map: Vec<(reqwest::Url, std::path::PathBuf)>) -> Result<()> {
    let download_progresses = indicatif::MultiProgress::new();
    let mut download_futures = Vec::new();
    let client = reqwest::Client::new();
    for (url, dest) in map {
        let mut response = client.get(url.clone()).send().await?;
        let content_length = response.content_length().unwrap_or(0);
        let download_progress = indicatif::ProgressBar::new(content_length);
        download_progress.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                .progress_chars("#>-"),
        );
        download_progress.set_position(0);
        download_progress.set_length(content_length);
        download_progress.set_draw_target(indicatif::ProgressDrawTarget::stderr());
        download_progress.set_message(format!("ダウンロード中: {}", url));
        download_progresses.add(download_progress.clone());

        let download_future = async move {
            let mut dest_file = tokio::fs::File::create(&dest).await?;
            while let Some(chunk) = response.chunk().await? {
                dest_file.write_all(&chunk).await?;
                download_progress.inc(chunk.len() as u64);
            }
            download_progress.finish_with_message("ダウンロード完了");
            Ok::<_, anyhow::Error>(())
        };
        download_futures.push(download_future);
    }

    let results = futures::future::join_all(download_futures).await;
    download_progresses.clear()?;

    results.into_iter().collect::<Result<_>>()
}
