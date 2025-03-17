use lzma_rs::lzma_decompress;
use std::{collections::HashMap, io::Read, sync::Arc};
use tap::Pipe;
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
    #[allow(clippy::option_env_unwrap)]
    option_env!("ZTS_DEVICE")
        .expect("デバイスが設定されていません")
        .parse()
        .expect("デバイスが不正です")
});

static VERSION: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    #[allow(clippy::option_env_unwrap)]
    option_env!("ZTS_VERSION")
        .expect("バージョンが設定されていません")
        .to_string()
});

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct HashInfo {
    position: u64,
    compressed_size: u64,
    decompressed_size: u64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseInfo {
    version: String,
    device: String,
    hashes: HashMap<String, String>,
    hash_info: HashMap<String, HashInfo>,
}

#[derive(Debug, Clone)]
struct ArchiveInfo {
    index: u32,
    url: reqwest::Url,
    size: u64,
}

#[derive(Debug, Clone)]
struct DownloadSpecification {
    url: reqwest::Url,
    dest: std::path::PathBuf,
    range: Option<(u64, u64)>,
}

static FILE_HASHES_NAME: &str = "file_hashes.json";

#[tokio::main]
async fn main() {
    if enable_ansi_support::enable_ansi_support().is_err() {
        colored::control::set_override(false);
    }
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

    let file_hashes_path = install_dir.join(FILE_HASHES_NAME);

    let (files_to_download, files_to_remove) = if file_hashes_path.exists() {
        info!("差分アップデートを行います。");
        diff_destination(install_dir, &release_info.hashes).await?
    } else {
        info!("新規インストールを行います。");
        (release_info.hashes.keys().cloned().collect(), vec![])
    };

    let (download_compressed_size, download_decompressed_size) =
        sum_download_size(&files_to_download, &release_info).await?;

    let mut remove_size = 0;
    for name in &files_to_remove {
        let stat = tokio::fs::metadata(install_dir.join(name.clone())).await?;
        remove_size += stat.len();
    }

    info!(
        "ダウンロードするファイルの容量：{}",
        indicatif::HumanBytes(download_compressed_size)
    );
    info!(
        "解凍時のディスク使用量：{}",
        indicatif::HumanBytes(download_decompressed_size)
    );
    if !files_to_remove.is_empty() {
        info!(
            "削除するファイルの容量：{}",
            indicatif::HumanBytes(remove_size)
        );
    }
    info!("インストール先：{}", install_dir.display());

    if !dialoguer::Confirm::new()
        .with_prompt("実行しますか？")
        .interact()?
    {
        return Ok(());
    }

    let mut archive_infos = release_assets
        .iter()
        .filter_map(|asset| {
            if !asset.name.contains(".bin.") {
                return None;
            }
            let url = asset.browser_download_url.clone();
            let number = asset
                .name
                .split('.')
                .nth_back(1)
                .unwrap()
                .parse::<u32>()
                .unwrap();
            let size = asset.size;
            Some(ArchiveInfo {
                index: number,
                url,
                size: size as u64,
            })
        })
        .collect::<Vec<_>>();
    archive_infos.sort_by_key(|info| info.index);

    info!("解凍中...");

    let download_root = install_dir.join("downloads");
    let downloaded_files = download_partitions(
        &archive_infos,
        &release_info.hash_info,
        &files_to_download,
        &download_root,
    )
    .await?;

    for name in files_to_remove {
        tokio::fs::remove_file(install_dir.join(name)).await?;
    }

    deploy_files(&downloaded_files, install_dir, &release_info.hashes).await?;
    for (_, downloaded_file) in downloaded_files {
        tokio::fs::remove_file(downloaded_file).await?;
    }

    let file_hashes = serde_json::to_vec(&release_info.hashes)?;
    tokio::fs::write(file_hashes_path, file_hashes).await?;

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

async fn download_urls(map: Vec<DownloadSpecification>) -> Result<()> {
    let download_progresses = indicatif::MultiProgress::new();
    let mut download_futures = Vec::new();
    let client = reqwest::Client::new();
    let throttle = Arc::new(tokio::sync::Semaphore::new(4));
    for spec in map {
        let mut response = client
            .get(spec.url.clone())
            .pipe(|req| {
                if let Some((bytes_start, bytes_length)) = spec.range {
                    req.header(
                        "Range",
                        format!("bytes={}-{}", bytes_start, bytes_start + bytes_length - 1),
                    )
                } else {
                    req
                }
            })
            .send()
            .await?;
        let content_length = response.content_length().unwrap_or(0);

        let download_future = {
            let download_progresses = download_progresses.clone();
            let throttle = throttle.clone();
            async move {
                let _permit = throttle.acquire().await.unwrap();
                let download_progress =
                    download_progresses.add(indicatif::ProgressBar::new(content_length));
                download_progress.set_style(
                    indicatif::ProgressStyle::default_bar()
                        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                        .progress_chars("#>-"),
                );
                download_progress.set_length(content_length);
                download_progress.set_message(format!("ダウンロード中: {}", spec.url));
                let mut dest_file = tokio::fs::File::create(&spec.dest).await?;
                while let Some(chunk) = response.chunk().await? {
                    dest_file.write_all(&chunk).await?;
                    download_progress.inc(chunk.len() as u64);
                }
                download_progress.finish();
                download_progresses.remove(&download_progress);
                Ok::<_, anyhow::Error>(())
            }
        };
        download_futures.push(download_future);
    }

    let results = futures::future::join_all(download_futures).await;
    download_progresses.clear()?;

    results.into_iter().collect::<Result<_>>()
}

async fn diff_destination(
    old_destination: &std::path::Path,
    files_to_hash: &HashMap<String, String>,
) -> Result<(Vec<String>, Vec<String>)> {
    let existing_file_hashes = tokio::fs::read(&old_destination.join(FILE_HASHES_NAME)).await?;
    let existing_file_hashes: HashMap<String, String> = serde_json::from_slice(&existing_file_hashes)?;
    let mut files_to_download = Vec::new();
    let mut files_to_remove = Vec::new();
    for (name, latest_hash) in files_to_hash {
        if existing_file_hashes
            .get(name)
            .is_none_or(|current_hash| latest_hash != current_hash)
        {
            files_to_download.push(name.clone());
        }
    }
    for name in existing_file_hashes.keys() {
        if files_to_hash.get(name).is_none() {
            files_to_remove.push(name.clone());
        }
    }
    Ok((files_to_download, files_to_remove))
}

async fn sum_download_size(
    files_to_download: &[String],
    release_info: &ReleaseInfo,
) -> Result<(u64, u64)> {
    let mut download_compressed_size = 0;
    let mut download_decompressed_size = 0;
    for name in files_to_download {
        let hash_info = release_info
            .hash_info
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("{}のハッシュ情報が見つかりませんでした", name))?;
        download_compressed_size += hash_info.compressed_size;
        download_decompressed_size += hash_info.decompressed_size;
    }
    Ok((download_compressed_size, download_decompressed_size))
}

async fn download_partitions(
    archive_infos: &[ArchiveInfo],
    hash_infos: &HashMap<String, HashInfo>,
    files_to_download: &[String],
    download_root: &std::path::Path,
) -> Result<HashMap<String, std::path::PathBuf>> {
    let download_partitions_map = vec![];
    let mut sum_sizes = vec![0; archive_infos.len()];
    let mut current_sum = 0;
    for archive_info in archive_infos {
        sum_sizes.push(current_sum);
        current_sum += archive_info.size;
    }

    let mut download_partition_map = vec![];
    let mut file_paths = HashMap::new();
    for hash in files_to_download {
        let hash_info = hash_infos
            .get(hash)
            .ok_or_else(|| anyhow::anyhow!("{}のハッシュ情報が見つかりませんでした", hash))?;

        let start_index = sum_sizes
            .binary_search(&hash_info.position)
            .unwrap_or_else(|x| x);

        let dest_path = download_root.join(format!("{}.lzma", hash));
        file_paths.insert(hash.clone(), dest_path.clone());
        download_partition_map.push(DownloadSpecification {
            url: archive_infos[start_index].url.clone(),
            dest: dest_path,
            range: Some((
                hash_info.position - sum_sizes[start_index],
                hash_info.compressed_size,
            )),
        });
    }

    download_urls(download_partitions_map).await?;

    Ok(file_paths)
}

async fn deploy_files(
    downloaded_files: &HashMap<String, std::path::PathBuf>,
    install_dir: &std::path::Path,
    file_to_hash: &HashMap<String, String>,
) -> Result<()> {
    let mut hash_to_files = HashMap::new();
    for (hash, path) in downloaded_files {
        hash_to_files
            .entry(file_to_hash[hash].clone())
            .or_insert_with(Vec::new)
            .push(path);
    }

    let progress = indicatif::ProgressBar::new(hash_to_files.len() as u64);
    let mut futures = Vec::new();
    let semaphore = Arc::new(tokio::sync::Semaphore::new(4));
    for (hash, paths) in hash_to_files {
        let dest_paths = paths
            .iter()
            .map(|path| install_dir.join(path))
            .collect::<Vec<_>>();
        let input_path = downloaded_files.get(&hash).unwrap().clone();
        let progress = progress.clone();

        let semaphore = semaphore.clone();
        futures.push(async move {
            let _permit = semaphore.acquire().await.unwrap();

            tokio::task::spawn_blocking(move || -> Result<()> {
                let input = std::fs::File::open(input_path)?;
                let mut input = std::io::BufReader::new(input);
                let mut output = std::fs::File::create(&dest_paths[0])?;

                lzma_decompress(&mut input, &mut output)?;
                progress.inc(1);
                for dest_path in dest_paths.iter().skip(1) {
                    std::fs::copy(&dest_paths[0], dest_path)?;
                }

                Ok(())
            })
            .await
            .map_err(anyhow::Error::from)
            .pipe(|result| match result {
                Ok(Ok(())) => Ok(()),
                Ok(Err(err)) => Err(err),
                Err(err) => Err(err),
            })
        });
    }

    futures::future::join_all(futures)
        .await
        .into_iter()
        .collect::<Result<()>>()?;

    progress.finish();

    Ok(())
}
