use std::io::Read;
use std::str::FromStr;
use tokio::io::AsyncWriteExt;
mod log;
use anyhow::Result;
use colored::Colorize;

#[derive(Debug, Copy, Clone, Default, PartialEq, strum::Display, strum::EnumString)]
enum Device {
    #[strum(to_string = "CPU", ascii_case_insensitive)]
    #[default]
    Cpu,
    #[strum(to_string = "CUDA", ascii_case_insensitive)]
    Cuda,
}

static DEVICE: std::sync::LazyLock<Device> = std::sync::LazyLock::new(|| {
    std::env::var("ZTS_DEVICE")
        .ok()
        .and_then(|device| Device::from_str(&device).ok())
        .expect("デバイスが設定されていません")
});

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
    let result = main_inner().await;
    if let Err(ref err) = result {
        error!("エラーが発生しました：{:#?}", err);
    }

    info!("Enterキーを押すと終了します...");
    let mut stdin = std::io::stdin();
    let _ = stdin.read(&mut [0u8]).unwrap();
    std::process::exit(result.is_err() as i32);
}

async fn main_inner() -> Result<()> {
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
    let device = *DEVICE;

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

    let szr_path = install_dir.join("7zr.exe");
    {
        let mut szr = reqwest::get("https://www.7-zip.org/a/7zr.exe").await?;
        let mut szr_file = tokio::fs::File::create(&szr_path).await?;
        while let Some(chunk) = szr.chunk().await? {
            szr_file.write_all(&chunk).await?;
        }
    }

    let download_map = release_assets
        .iter()
        .filter_map(|asset| {
            if !asset.name.contains(".7z.") {
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
    command.arg(format!("-o{}", install_dir.display()));
    let first_7z = release_assets
        .iter()
        .find(|asset| asset.name.contains(".7z."))
        .ok_or_else(|| anyhow::anyhow!("7zファイルが見つかりませんでした"))?;
    command.arg(install_dir.join(first_7z.name.clone()));
    command.stdout(std::process::Stdio::inherit());
    command.stderr(std::process::Stdio::inherit());

    let status = command.status().await?;

    if !status.success() {
        return Err(anyhow::anyhow!("7zrの実行に失敗しました"));
    }

    tokio::fs::remove_file(szr_path).await?;
    for asset in release_assets {
        if !asset.name.contains(".7z.") {
            continue;
        }
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
    let mut assets = assets
        .into_iter()
        .filter(|asset| {
            asset.name.contains("windows")
                && asset.name.contains(&device.to_string().to_lowercase())
        })
        .collect::<Vec<_>>();
    assets.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(assets)
}

async fn download_urls(map: Vec<(reqwest::Url, std::path::PathBuf)>) -> Result<()> {
    let download_progresses = indicatif::MultiProgress::new();
    let mut download_futures = Vec::new();
    let client = reqwest::Client::new();
    for (url, dest) in map {
        let mut response = client.get(url.clone()).send().await?;
        let content_length = response.content_length().unwrap_or(0);
        let download_progress =
            download_progresses.add(indicatif::ProgressBar::new(content_length));
        download_progress.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                .progress_chars("#>-"),
        );
        download_progress.set_length(content_length);
        download_progress.set_message(format!("ダウンロード中: {}", url));

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
