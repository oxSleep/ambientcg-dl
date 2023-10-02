use dialoguer::Select;
use anyhow::Context;
use zip;
use reqwest::Client;
use serde_json::Value;
use std::fs::{self, File};
use std::io::Write;

#[derive(Debug)]
pub struct FoundAsset {
    pub assetid: String,
    pub datatype: String,
    pub category: String,
    pub file: String,
}

pub struct PromptData {
    pub assettype: String,
    quality: String,
}

pub fn prompt_user() -> PromptData {

    let asset_type = vec![
        "Atlas",
        "Brush",
        "Decal",
        "HDRI",
        "Material",
        "PlainTexture",
        "Substance",
    ];


    let asset_type_selection = Select::new()
        .with_prompt("Wich type of asset do you want to download ? ")
        .items(&asset_type)
        .interact().unwrap();

    let mut initial_quality = vec!["1K", "2K", "4K", "8K"];
    let mut quality_selection: usize = 0;

    if asset_type[asset_type_selection] != "Substance"{
        if asset_type[asset_type_selection] == "HDRI" {
            initial_quality.push("12K");
            initial_quality.push("16K");
        } 
        quality_selection = Select::new()
            .with_prompt("In wich quality would you like to download the assets ? ")
            .items(&initial_quality)
            .interact().unwrap();


    };


    let prompt: PromptData = PromptData { assettype: asset_type[asset_type_selection].to_string(), quality: initial_quality[quality_selection].to_string() };
    return prompt;

}

pub async fn get_number_of_asset(client: &Client, asset_type: &String) -> Result<u32, anyhow::Error> {
    let url = format!("https://ambientcg.com/api/v2/full_json?type={}", asset_type);

    let response = client.get(url).send().await?;
    let text = response.text().await?;

    let parsed_response: serde_json::Value = serde_json::from_str(&text)?;

    let numberofasset = parsed_response["numberOfResults"].to_string();

    Ok(numberofasset.parse::<u32>()?)


}

pub async fn get_link(client: &Client, offset: i32, settings: &PromptData) -> Result<FoundAsset, anyhow::Error> {

    let api_url = format!("https://ambientCG.com/api/v2/full_json?limit=1&type={}&offset={}", settings.assettype, offset);
    let response = client.get(api_url).send().await?;
    let text = response.text().await?;

    let parsed_response: serde_json::Value = serde_json::from_str(&text)?;


    let assetid = match_parse(&parsed_response, "assetId".to_string())?;
    let datatype = match_parse(&parsed_response, "dataType".to_string())?;
    let category = match_parse(&parsed_response, "category".to_string())?;
    let file;

    if datatype == "HDRI" {
        file = format!("{}_{}-HDR.exr", assetid, settings.quality);
    } else if  datatype == "Substance" {
        file = format!("{}.sbsar", assetid);

    } else {
        file = format!("{}_{}-PNG.zip", assetid, settings.quality);
    }

    let asset: FoundAsset = FoundAsset {
        assetid,
        datatype,
        category,
        file,
    };


    //println!("Asset fetched: {:#?}", &asset.assetid);
    return Ok(asset);
}

fn match_parse(text: &Value, search: String) -> Result<String, anyhow::Error> {
    let result = text["foundAssets"][0][&search].as_str()
        .ok_or_else(|| anyhow::anyhow!("failed to extract '{}' from JSON", &search))?;
    Ok(result.to_string())
}

pub async fn download_file(client: &Client, asset: &FoundAsset) -> Result<(), anyhow::Error> {


    let dl_url = format!("https://ambientCG.com/get?file={}", asset.file);
    let outpout_path = format!("./ambientCG/{}/{}/{}", asset.datatype, asset.category, asset.file);

    let res = client.get(dl_url).send().await?;
    let _ = create_folder_if_not_exists(&asset.category, &asset.datatype);

    let mut output_file = File::create(outpout_path)?;
    let _ = output_file.write_all(&res.bytes().await?);
    //println!("{} downloaded successfully", asset.file);
    Ok(())

}

fn create_folder_if_not_exists(category: &String, datatype: &String) -> Result<(), anyhow::Error> {
    let folder_path = format!("./ambientCG/{}/{}", datatype, category);
    let fp = folder_path.as_str();
    if !fs::metadata(fp).is_ok() {
        fs::create_dir_all(fp)?;
    }
    //println!("Folder created or already exists: {}", &folder_path);
    Ok(())
}

pub fn unzip_file(asset: &FoundAsset) -> Result<(), anyhow::Error> {

    let file_path = format!("./ambientCG/{}/{}/{}", asset.datatype, asset.category, asset.file);

    if asset.file.ends_with(".zip") {
        let file = fs::File::open(&file_path)
            .with_context(|| format!("Failed to open file: {}", &file_path))?;


        let mut archive = zip::ZipArchive::new(&file)
            .with_context(|| format!("Failed to create ZipArchive for file: {}", &file_path))?;

        let _ = archive.extract(&file_path.trim_end_matches(".zip"))
            .with_context(|| format!("Failed to extract archive {:?} to {}", &file, &file_path))?;

        let _ = fs::remove_file(&file_path)
            .with_context(|| format!("Failed to remove file: {}", &file_path))?;
    }
    Ok(())
}

