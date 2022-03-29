use crate::config::{load_config, Config, LoadConfigErr};
use crate::license::{read_license, License, ReadLicenseErr, AddToFileResult};

use ignore::Walk;

use colored::*;

pub fn execute() {
  let config: Config;
    match load_config() {
        Ok(cfg) => config = cfg,
        Err(e) => match e {
            LoadConfigErr::JsonFormattingErr => {
                println!("Error: Your config file wasn't formatted correctly.");
                std::process::exit(exitcode::CONFIG);
            }
            LoadConfigErr::CreateDefaultConfigErr => {
                println!("Error: Failed to create default config file.");
                std::process::exit(exitcode::IOERR)
            }
            LoadConfigErr::LoadUserConfigErr => {
                println!("Error: failed to load user config file.");
                std::process::exit(exitcode::IOERR)
            }
            LoadConfigErr::NotFoundErr => std::process::exit(exitcode::IOERR),
        },
    };

    let license: License;

    match read_license() {
        Ok(l) => license = l,
        Err(e) => match e {
            ReadLicenseErr::FileReadErr => {
                println!("Error: Couldn't find a .licensesnip file in the current working directory's root.");
                std::process::exit(exitcode::CONFIG)
            }
        },
    }

    let filetype_map = config.get_filetype_map();

    let mut changed_files_count: u32 = 0;
    let mut matched_filetypes_count: u32 = 0;

    for result in Walk::new("./") {
        // Each item yielded by the iterator is either a directory entry or an
        // error, so either print the path or the error.
        match result {
            Ok(entry) => (|entry: ignore::DirEntry| {
                match entry.file_type() {
                    Some(t) => {
                        if !t.is_file() {
                            return;
                        }
                    }
                    None => return,
                }

                // Get file extension
                let file_name = entry.file_name().to_string_lossy();
                let ext;
                match file_name.split(".").last() {
                    Some(e) => ext = e,
                    None => return,
                }

                let filetype_cfg = match filetype_map.get(ext) {
                    Some(e) => {
                      matched_filetypes_count += 1;
                      e
                    },
                    None => {
                        // No configuration for this file type
                        return;
                    }
                };

                if !filetype_cfg.enable {
                  // Disabled for this filetype
                  return;
                }

                let raw_lines = license.get_lines();

                let f_lines = License::get_formatted_lines(&raw_lines, &file_name, 2022);

                let header_text = License::get_header_text(&f_lines, filetype_cfg);
              
                match License::add_to_file(&entry, &header_text) {
                    Ok(r) => {
                        match r {
                            AddToFileResult::Added => {
                                changed_files_count += 1;
                            }
                            _ => {}
                        };
                    }
                    Err(e) => {
                        println!("{:?}", e)
                    }
                }
            })(entry),
            Err(err) => println!("ERROR: {}", err),
        }
    }

    let status_str = format!("✔ Added license header to {} files.", changed_files_count);
    let status_str_colored = status_str.green();

    println!("{}", status_str_colored);

    if matched_filetypes_count == 0 {
      let warning = format!("{}\n\n{}\n\n{}", "⚠ No supported file types were found. You may need to add styling rules for your filetypes in your user/local config file. Run".yellow(), "licensesnip help", "for more info.".yellow());

      println!("{}", warning);
      
    }

    std::process::exit(exitcode::OK);
}