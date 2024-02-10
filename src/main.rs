use ambientcg_dl::*;
use indicatif::ProgressStyle;
use indicatif::ProgressBar;

const USER_AGENT: &str = "Mozilla/5.0 (Linux; Android 10; AQM-LX1; HMSCore 6.11.0.331) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.0.4844.88 HuaweiBrowser/14.0.0.322 Mobile Safari/537.36";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {

    let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
    let settings = prompt_user();
    let nb = get_number_of_asset(&client, &settings.assettype, &settings.query).await?;

    let pb = ProgressBar::new(nb.into());
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{msg}] [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len}")
        .unwrap()
        .progress_chars("#>-"));


    let mut init_offset = 0;
    for _ in 0..nb {
        let asset = get_link(&client, init_offset, &settings).await?;
        pb.set_message(format!("Downloading asset: {}", &asset.assetid));


        if  check_download_link(&client, &asset.file).await? {
            download_file(&client, &asset).await?;
            let _ = unzip_file(&asset);
        } else {
            println!("Download Unavailable, skipping...");
        }
        pb.inc(1);
        init_offset += 1;

    }


    pb.finish_with_message("Process completed successfully");
    Ok(())
}
