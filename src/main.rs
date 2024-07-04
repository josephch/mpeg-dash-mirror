use clap::Parser;

pub mod mpd;

fn download(url: &str, path: &std::path::Path) {
    let parent = path.parent();
    if let Some(directory) = parent {
        match std::fs::create_dir_all(directory) {
            Ok(_) => {
                let result = reqwest::blocking::get(url);
                match result {
                    Ok(response) => match response.bytes() {
                        Ok(bytes) => match std::fs::write(path, bytes) {
                            Ok(_) => println!("downloaded  url {}", url),
                            Err(e) => eprintln!("HTTP get failure : url {} error {}", url, e),
                        },
                        Err(_) => todo!(),
                    },
                    Err(e) => eprintln!("HTTP get failure : {}", e),
                }
            }
            Err(e) => eprintln!("Could not create parent directory {}", e),
        }
    } else {
        eprintln!("Could not get parent directory");
    }
}

#[derive(clap::Parser, Debug)]
struct CommandLineArgs {
    /// Output folder to store files
    #[arg(short, long, default_value_t = {"harvest".to_string()})]
    output_directory: String,
    #[arg(long)]
    url: String,
}

fn main() {
    let args = CommandLineArgs::parse();
    let url = args.url;
    println!("url {}", url);
    let mut manifest_path_str = args.output_directory.clone();
    manifest_path_str.push(std::path::MAIN_SEPARATOR);
    manifest_path_str.push_str("manifest.mpd");
    let manifest_path = std::path::Path::new(&manifest_path_str);
    download(&url, manifest_path);

    match std::fs::read_to_string(manifest_path) {
        Ok(manifest_text) => match mpd::get_fragment_urls(manifest_text, &url) {
            Some(url_info) => {
                let iterator = url_info.urls.iter();
                for (url_idx, url) in iterator.enumerate() {
                    if url.starts_with(&url_info.base_url) {
                        let mut path_str = args.output_directory.clone();
                        path_str.push(std::path::MAIN_SEPARATOR);
                        path_str.push_str(&url[url_info.base_url.len()..]);
                        let end = path_str.find("?").unwrap_or(path_str.len());
                        path_str.truncate(end);
                        let path = std::path::Path::new(&path_str);
                        if path.exists() {
                            println!(
                                "Segment {} url {} path {} exists, skip",
                                url_idx, url, path_str
                            );
                        } else {
                            download(url, path);
                        }
                    } else {
                        println!(
                            "Segment {} url {} is not start with base_url {}",
                            url_idx, url, url_info.base_url
                        );
                    }
                }
            }
            None => {
                eprintln!("fragement urls not available")
            }
        },
        Err(e) => {
            println!("Error: reading manifest {}", e);
        }
    }
}
