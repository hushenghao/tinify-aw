use std::{error::Error, path::Path};

use rexiv2::Rexiv2Error;
use tinify::{Tinify, TinifyError};
use walkdir::{DirEntry, WalkDir};

#[warn(dead_code)]
const LOG_TAG: &str = "Tinify";

const TAG_VALUE: &str = "tinify";
const TAG_IMAGE_DESCRIPTION: &str = "Exif.Image.ImageDescription";
// const TAG_USER_COMMENT: &str = "Exif.Photo.UserComment";

const KEY_TINIFY: &str = "xxxxxx";

fn main() {
    let input_path = "/Users/dede/Downloads/test";
    // let input_path = "/Users/dede/Downloads/draw_text.webp";
    // let input_path = "/Users/dede/Downloads/南京地铁建设规划2035线路图-2022b v1.8©宁东萝卜.jpg";
    // let input_path = "/Users/dede/Downloads/WechatIMG62 (1).jpeg";
    // let input_path = "/Users/dede/Downloads/image-2022-12-22-11-05-06-329.png";

    rexiv2::initialize().expect("Unable to initialize rexiv2");

    for image in collection(input_path) {
        log("===============");
        let path = image.path();
        log(format!("读取'tinify'标记: {}", path.display()).as_str());
        let result = is_marked(path);
        let is_marked = match result {
            Ok(b) => b,
            Err(e) => {
                loge("读取'tinify'错误, 继续压缩!", &e);
                false
            }
        };
        if is_marked {
            log("读取到'tinify'标记, 不需要再次压缩.");
            continue;
        } else {
            log("未读取到'tinify'标记, 需要压缩.");
        }

        log(format!("开始压缩: {}", path.display()).as_str());
        let result = tinify(KEY_TINIFY, path, path);
        match result {
            Ok(_) => log("压缩成功!"),
            Err(e) => {
                loge(format!("压缩错误: {}", path.display()).as_str(), &e);
                continue;
            }
        }

        let result = mark(path);
        match result {
            Ok(_) => log("写入'tinify'标记, 下次不再压缩."),
            Err(e) => loge(format!("写入'tinify'标记错误!").as_str(), &e),
        }
    }
}

fn collection(path: &str) -> Vec<DirEntry> {
    fn filter_target_file(entry: &DirEntry, target_suffixs: &[&str]) -> bool {
        if entry.path().is_dir() {
            return true;
        }
        if entry.path().is_symlink() {
            return false;
        }
        return entry
            .file_name()
            .to_str()
            .map(|s| s.to_lowercase())
            .map(|s| {
                if s.starts_with(".") || s.ends_with(".9.png") {
                    // 排除隐藏文件或.9文件
                    return false;
                }
                for suffix in target_suffixs {
                    if s.ends_with(suffix) {
                        return true;
                    }
                }
                return false;
            })
            .unwrap_or(false);
    }

    let suffixs = [".jpg", ".jpeg", ".png", ".webp"];

    return WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| filter_target_file(e, &suffixs))
        .filter_map(|f| f.ok())
        .filter(|e| e.path().is_file())
        .collect();
}

fn tinify(key: &str, from: &Path, to: &Path) -> Result<(), TinifyError> {
    let tinify = Tinify::new().set_key(key);
    let client = tinify.get_client()?;
    let _ = client.from_file(from)?.to_file(to)?;
    Ok(())
}

fn mark(image: &Path) -> Result<(), Rexiv2Error> {
    let meta = rexiv2::Metadata::new_from_path(image)?;
    // let meta = match result {
    //     Ok(meta) => meta,
    //     Err(e) => return Err(e),
    // };
    meta.clear();
    let result = meta.set_tag_string(TAG_IMAGE_DESCRIPTION, TAG_VALUE);
    return match result {
        Err(e) => Err(e),
        Ok(_) => match meta.save_to_file(image) {
            Err(e) => Err(e),
            _ => Ok(()),
        },
    };
}

fn is_marked(image: &Path) -> Result<bool, Rexiv2Error> {
    let meta = rexiv2::Metadata::new_from_path(image)?;
    let result = meta.get_tag_string(TAG_IMAGE_DESCRIPTION);
    return match result {
        Err(e) => match e {
            Rexiv2Error::NoValue => Ok(false),
            _ => Err(e),
        },
        _ => Ok(true),
    };
}

#[warn(dead_code)]
fn log(msg: &str) {
    println!("{}: {}", LOG_TAG, msg)
}

#[warn(dead_code)]
fn loge(msg: &str, e: &dyn Error) {
    eprintln!("{}: {}: {}", LOG_TAG, msg, e)
}
