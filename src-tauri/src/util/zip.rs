use std::fs::{self, File};
use std::path::Path;
use zip::ZipArchive;

pub fn unzip_file(zip_path: &Path, dest: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| format!("打开文件失败: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("解析文件失败,{}", e))?;

    for i in 0..archive.len() {
        let mut zip_file = archive
            .by_index(i)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?;
        let outpath = dest.join(zip_file.sanitized_name());

        if zip_file.name().ends_with('/') {
            // 是目录
            fs::create_dir_all(&outpath).map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            // 是文件
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("创建文件目录失败: {}", e))?;
            }
            let mut outfile = File::create(&outpath).map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut zip_file, &mut outfile)
                .map_err(|e| format!("写入解压文件失败: {}", e))?;
        }
    }

    Ok(())
}
