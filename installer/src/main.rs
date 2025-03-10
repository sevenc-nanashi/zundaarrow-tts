mod log;
use strum::IntoEnumIterator;
use clap::Parser;
use colored::Colorize;

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

static VERSION: Option<&'static str> = option_env!("ZTS_VERSION");

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();
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
    let Some(version) = VERSION else {
        error!("Version information is not available.");
        panic!();
    };
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

    info!("バージョン：{}", version);
    info!("ハードウェア：{:?}", device);

    let release_assets = get_release_assets(version, device).await;
}

async fn get_release_assets(
    version: &str,
    device: Device,
) -> Result<Vec<octocrab::models::repos::Asset>, Box<dyn std::error::Error>> {
    info!("ファイル一覧を取得中...");
    let assets = octocrab::instance()
        .repos("sevenc-nanashi", "zundaarrow-tts")
        .releases()
        .get_by_tag(version)
        .await?
        .assets;
    Ok(assets
        .into_iter()
        .filter(|asset| asset.name.ends_with("zip"))
        .collect())
}
