mod log;
use anyhow::{Context, Result};
use colored::Colorize;
use itertools::Itertools;
use lzma::LzmaReader;
use std::{
    collections::{HashMap, HashSet},
    io::Read,
    sync::Arc,
};
use tap::Pipe;
use tokio::io::AsyncWriteExt;

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
    hashes: HashMap<String, FileHash>,
    hash_info: HashMap<FileHash, HashInfo>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
struct FileHash(String);
impl std::fmt::Display for FileHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[derive(Debug, Clone)]
struct ArchiveInfo {
    index: u32,
    url: reqwest::Url,
    size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DownloadRange {
    start: u64,
    size: u64,
}

#[derive(Debug, Clone)]
struct DownloadSpecification {
    url: reqwest::Url,
    dest: std::path::PathBuf,
    range: Option<DownloadRange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OptimizedDownloadSpecification {
    url: reqwest::Url,
    dest: Vec<(std::path::PathBuf, u64)>,
    range: Option<DownloadRange>,
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
        .find(|asset| asset.name.ends_with(".json"))
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
                .nth_back(0)
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

    info!("ダウンロード中...");

    let download_root = install_dir.join("downloads");
    tokio::fs::create_dir_all(&download_root).await?;
    let downloaded_files = download_partitions(
        &archive_infos,
        &release_info.hashes,
        &release_info.hash_info,
        &files_to_download,
        &download_root,
    )
    .await?;

    for name in files_to_remove {
        tokio::fs::remove_file(install_dir.join(name)).await?;
    }

    info!("ファイルを展開中...");
    deploy_files(
        &files_to_download,
        &downloaded_files,
        install_dir,
        &release_info.hashes,
    )
    .await?;
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

async fn download_urls(download_specs: &[DownloadSpecification]) -> Result<()> {
    let optimized_specs = optimize_specs(download_specs);

    let download_progresses = indicatif::MultiProgress::new();
    let mut download_futures = Vec::new();
    let throttle = Arc::new(tokio::sync::Semaphore::new(8));

    let client = Arc::new(reqwest::Client::new());

    let all_download_progress =
        download_progresses.add(indicatif::ProgressBar::new(download_specs.len() as u64));
    all_download_progress.set_style(
        indicatif::ProgressStyle::default_bar()
            .template(
                "{spinner:.blue} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )?
            .progress_chars("#>-"),
    );
    for spec in optimized_specs {
        let download_future = {
            let download_progresses = download_progresses.clone();
            let all_download_progress = all_download_progress.clone();
            let throttle = throttle.clone();
            let client = client.clone();
            async move {
                let _permit = throttle.acquire().await.unwrap();

                let download_progress = download_progresses.add(indicatif::ProgressBar::new(
                    spec.range.map(|r| r.size).unwrap_or(0),
                ));
                download_progress.set_style(
                    indicatif::ProgressStyle::default_bar()
                        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.green/cyan}] {bytes}/{total_bytes} ({eta})")?
                        .progress_chars("#>-"),
                );
                let mut response = client
                    .get(spec.url.clone())
                    .pipe(|req| {
                        if let Some(range) = spec.range {
                            req.header(
                                "Range",
                                format!("bytes={}-{}", range.start, range.start + range.size - 1),
                            )
                        } else {
                            req
                        }
                    })
                    .send()
                    .await?;
                if !response.status().is_success() {
                    anyhow::bail!("ダウンロードに失敗しました：{}", response.status());
                }
                if let Some(content_length) = response.content_length() {
                    download_progress.set_length(content_length);
                }
                assert_eq!(spec.dest[0].1, 0);
                let mut dest_file = tokio::fs::File::create(&spec.dest[0].0).await?;
                let mut current_dest_index = 0;
                let mut current_bytes = 0;
                while let Some(mut chunk) = response.chunk().await? {
                    while spec
                        .dest
                        .get(current_dest_index + 1)
                        .map(|(_, start)| *start)
                        .is_some_and(|start| current_bytes + chunk.len() as u64 >= start)
                    {
                        let next_dest_start_bytes = spec.dest[current_dest_index + 1].1;
                        let part = chunk.split_to((next_dest_start_bytes - current_bytes) as usize);

                        download_progress.inc(part.len() as u64);
                        dest_file.write_all(&part).await?;

                        all_download_progress.inc(1);
                        dest_file =
                            tokio::fs::File::create(&spec.dest[current_dest_index + 1].0).await?;
                        current_dest_index += 1;
                        current_bytes = next_dest_start_bytes;
                    }

                    download_progress.inc(chunk.len() as u64);
                    dest_file.write_all(&chunk).await?;
                    current_bytes += chunk.len() as u64;
                }
                all_download_progress.inc(1);
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
    files_to_hash: &HashMap<String, FileHash>,
) -> Result<(Vec<String>, Vec<String>)> {
    let existing_file_hashes = tokio::fs::read(&old_destination.join(FILE_HASHES_NAME)).await?;
    let existing_file_hashes: HashMap<String, FileHash> =
        serde_json::from_slice(&existing_file_hashes)?;
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
        let hash = release_info
            .hashes
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("{}のハッシュが見つかりませんでした", name))?;
        let hash_info = release_info
            .hash_info
            .get(hash)
            .ok_or_else(|| anyhow::anyhow!("{}のハッシュ情報が見つかりませんでした", name))?;
        download_compressed_size += hash_info.compressed_size;
        download_decompressed_size += hash_info.decompressed_size;
    }
    Ok((download_compressed_size, download_decompressed_size))
}

async fn download_partitions(
    archive_infos: &[ArchiveInfo],
    file_to_hash: &HashMap<String, FileHash>,
    hash_infos: &HashMap<FileHash, HashInfo>,
    files_to_download: &[String],
    download_root: &std::path::Path,
) -> Result<HashMap<FileHash, std::path::PathBuf>> {
    let mut sum_sizes = vec![];
    let mut current_sum = 0;
    for archive_info in archive_infos {
        sum_sizes.push(current_sum);
        current_sum += archive_info.size;
    }

    let mut download_specifications = vec![];
    let mut file_paths = HashMap::new();
    let hashes = files_to_download
        .iter()
        .map(|name| {
            file_to_hash
                .get(name)
                .ok_or_else(|| anyhow::anyhow!("{}のハッシュが見つかりませんでした", name))
        })
        .collect::<Result<HashSet<_>>>()?;
    for hash in hashes {
        let hash_info = hash_infos
            .get(hash)
            .ok_or_else(|| anyhow::anyhow!("{}のハッシュ情報が見つかりませんでした", hash))?;

        let start_index = sum_sizes
            .iter()
            .enumerate()
            .filter_map(|(index, sum)| {
                if hash_info.position >= *sum {
                    Some(index)
                } else {
                    None
                }
            })
            .last()
            .ok_or_else(|| anyhow::anyhow!("{}のアーカイブが見つかりませんでした", hash))?;

        let dest_path = download_root.join(format!("{}.lzma", hash));
        file_paths.insert(hash.clone(), dest_path.clone());
        download_specifications.push(DownloadSpecification {
            url: archive_infos[start_index].url.clone(),
            dest: dest_path,
            range: Some(DownloadRange {
                start: hash_info.position - sum_sizes[start_index],
                size: hash_info.compressed_size,
            }),
        });
    }

    download_urls(&download_specifications).await?;

    Ok(file_paths)
}

async fn deploy_files(
    files_to_download: &[String],
    downloaded_files: &HashMap<FileHash, std::path::PathBuf>,
    install_dir: &std::path::Path,
    file_to_hash: &HashMap<String, FileHash>,
) -> Result<()> {
    let mut hash_to_files = HashMap::new();
    for (file, hash) in file_to_hash {
        if !files_to_download.contains(file) {
            continue;
        }
        hash_to_files
            .entry(hash)
            .or_insert_with(Vec::new)
            .push(file.clone());
    }

    let progress = indicatif::ProgressBar::new(files_to_download.len() as u64).with_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.blue} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}")?
            .progress_chars("#>-"),
    );

    let mut futures = Vec::new();
    let semaphore = Arc::new(tokio::sync::Semaphore::new(4));
    for (hash, paths) in hash_to_files {
        let hash = hash.clone();
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
                let input = std::io::BufReader::new(input);
                std::fs::create_dir_all(dest_paths[0].parent().unwrap())?;

                let mut input = LzmaReader::new_decompressor(input)
                    .with_context(|| format!("{}の解凍に失敗しました", hash))?;
                let mut output = std::fs::File::create(&dest_paths[0])?;
                std::io::copy(&mut input, &mut output)?;
                progress.inc(1);
                for dest_path in dest_paths.iter().skip(1) {
                    std::fs::create_dir_all(dest_path.parent().unwrap())?;
                    std::fs::copy(&dest_paths[0], dest_path)?;

                    progress.inc(1);
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

fn optimize_specs(download_specs: &[DownloadSpecification]) -> Vec<OptimizedDownloadSpecification> {
    let grouped = download_specs
        .iter()
        .into_group_map_by(|spec| spec.url.clone());

    grouped
        .into_iter()
        .flat_map(|(url, specs)| {
            let mut dests = Vec::new();
            let sorted_specs = specs
                .into_iter()
                .sorted_by_key(|spec| spec.range.map_or(0, |r| r.start))
                .collect::<Vec<_>>();

            let full_specs = sorted_specs
                .iter()
                .filter(|spec| spec.range.is_none())
                .collect::<Vec<_>>();
            dests.extend(
                full_specs
                    .iter()
                    .map(|spec| OptimizedDownloadSpecification {
                        url: url.clone(),
                        dest: vec![(spec.dest.clone(), 0)],
                        range: None,
                    }),
            );
            let mut range_specs = sorted_specs
                .iter()
                .filter(|spec| spec.range.is_some())
                .collect::<Vec<_>>();

            while !range_specs.is_empty() {
                let range = range_specs[0].range.unwrap();
                let mut dest = vec![(range_specs[0].dest.clone(), 0)];
                range_specs.remove(0);
                let mut current_size = range.size;
                while !range_specs.is_empty() {
                    let next_range = range_specs[0].range.unwrap();
                    if range.start + current_size == next_range.start {
                        dest.push((range_specs[0].dest.clone(), current_size));
                        current_size += next_range.size;
                        range_specs.remove(0);
                    } else {
                        break;
                    }
                }
                dests.push(OptimizedDownloadSpecification {
                    url: url.clone(),
                    dest,
                    range: Some(DownloadRange {
                        start: range.start,
                        size: current_size,
                    }),
                });
            }

            dests
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_optimize_specs() {
        use super::{DownloadRange, DownloadSpecification, OptimizedDownloadSpecification};
        let specs = vec![
            DownloadSpecification {
                url: "https://example.com/1".parse().unwrap(),
                dest: "1".into(),
                range: Some(DownloadRange { start: 0, size: 10 }),
            },
            DownloadSpecification {
                url: "https://example.com/1".parse().unwrap(),
                dest: "2".into(),
                range: Some(DownloadRange {
                    start: 10,
                    size: 10,
                }),
            },
            DownloadSpecification {
                url: "https://example.com/1".parse().unwrap(),
                dest: "3".into(),
                range: Some(DownloadRange {
                    start: 30,
                    size: 10,
                }),
            },
            DownloadSpecification {
                url: "https://example.com/2".parse().unwrap(),
                dest: "4".into(),
                range: Some(DownloadRange {
                    start: 40,
                    size: 10,
                }),
            },
        ];
        let optimized = optimize_specs(&specs);
        assert_eq!(
            optimized,
            vec![
                OptimizedDownloadSpecification {
                    url: "https://example.com/1".parse().unwrap(),
                    dest: vec![("1".into(), 0), ("2".into(), 10)],
                    range: Some(DownloadRange { start: 0, size: 20 }),
                },
                OptimizedDownloadSpecification {
                    url: "https://example.com/1".parse().unwrap(),
                    dest: vec![("3".into(), 0)],
                    range: Some(DownloadRange {
                        start: 30,
                        size: 10,
                    }),
                },
                OptimizedDownloadSpecification {
                    url: "https://example.com/2".parse().unwrap(),
                    dest: vec![("4".into(), 0)],
                    range: Some(DownloadRange {
                        start: 40,
                        size: 10,
                    }),
                },
            ]
        );
    }
}
