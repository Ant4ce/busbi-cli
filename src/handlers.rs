use std::{env::consts, fs::{File, create_dir_all}};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::arguments::parse_directories;
use crate::help_msg::{HelpMessage};
use crate::boilerplates::{make_file_boilerplate, start_boilerplate, end_boilerplate, executable_boilerplate};

pub fn d_flag_handler(target_os: &str, execute: bool, source_files: Vec<PathBuf>, destination: &str) -> Result<String, HelpMessage> {
    let mut grande_string : String = String::new();
    for current_path in source_files {
        if current_path.is_dir() {
            let mut list_files : Vec<PathBuf> = Vec::new();

            list_files = match parse_directories(list_files, &current_path) {
                Ok(x) => x,
                Err(e) => return Err(e),
            };
            match d_flag_handler(target_os, execute, list_files, destination) {
                Ok(x) => {
                            println!("Success."); 
                            grande_string.push_str(&x);
                        },
                Err(e) => {eprintln!("Failed to process files/directories. Got error : {:?}", e); return Err(HelpMessage::FailedRecursionFS)},
            }
        } else {
            let file_content = make_file_boilerplate(target_os, &current_path, destination, true, false);
            grande_string.push_str(&file_content);

        }
    }
    Ok(grande_string)
} 

pub fn file_handler(target_os: &str, execute: bool, source_files: Vec<PathBuf>, destination: &str, d_flag: bool, m_flag: bool, close_window : bool) -> Result<(), HelpMessage> {

    for current_path in source_files {
        if current_path.is_dir() {
            let mut list_files : Vec<PathBuf> = Vec::new();
            list_files = match parse_directories(list_files, &current_path) {
                Ok(x) => x,
                Err(e) => return Err(e),
            };
            file_handler(target_os, execute, list_files, destination, d_flag, m_flag, close_window)?;
        } else {

            let mut my_path : PathBuf = PathBuf::new();
            println!("current OS is: {}", consts::OS);
            if consts::OS == "windows" {
                my_path = PathBuf::from(format!("{}\\{}", destination, &current_path.display()));
            } else if consts::OS == "linux" || consts::OS == "macos" {
                my_path = PathBuf::from(format!("{}/{}", destination, &current_path.display()));
            }
            let parent_path : &Path = &my_path.parent().unwrap();
            //Changes the file extension of what it was to '.txt', which the BadUSB format requires.
            let txt_path : PathBuf = PathBuf::from(PathBuf::from(&my_path.file_name().unwrap()).file_stem().unwrap()).with_extension("txt"); 
            //Rejoins parent and new file names, after change to file name extension.
            let joined_path : PathBuf = parent_path.join(&txt_path);
            let new_file = match file_nested_dirs(&joined_path) {
                Ok(file) => file,
                Err(e) => {eprintln!("Got error message: {:?}", e); return Err(e)},
            };
            let mut write_buf = BufWriter::with_capacity(1000000, new_file);

            let start_boiler: String = start_boilerplate(target_os, true, destination );
            let _ = write_buf.write(start_boiler.as_bytes());

            let file_content : String = make_file_boilerplate(target_os, &current_path, destination, false, true);
            let _ = write_buf.write(file_content.as_bytes());

            if execute {
                // Here I use &current path instead of x_value, I do this because on -m flag it makes no sense
                // to specify a file name for all of the new files to execute, so instead each file
                // will execute the file it creates. 
                let execute_boiler : String = match executable_boilerplate(target_os, &current_path, destination, d_flag, m_flag) {
                    Ok(x) => x,
                    Err(e) => panic!("Got an error: {:?}",e),
                };
                let _ = write_buf.write(execute_boiler.as_bytes());
            }

            let end_boiler: String = end_boilerplate(target_os, close_window);
            let _ = write_buf.write(end_boiler.as_bytes());

            //This pushes the contents of the buffer to the file. 
            match write_buf.flush() {
                Ok(_x) => println!("Successfully wrote file."),
                Err(e) => {eprintln!("got error: {}", e); return Err(HelpMessage::BufferFlushFailed)} 
            };
        }
    }
    Ok(())
}

fn file_nested_dirs(my_path: &PathBuf) -> Result<File, HelpMessage> {
    let parent_path : &Path = match my_path.parent() {
        Some(x) => x,
        None => return Err(HelpMessage::NoParentPath),
    };
    match create_dir_all(parent_path) {
        Ok(_) => println!("made directory. Continuing..."),
        Err(e) => {eprintln!("Failed to make directories, got error: {}", e); return Err(HelpMessage::FailedMakingDirs)},
    };
    match File::create(my_path) {
        Ok(file) => Ok(file),
        Err(_e) => Err(HelpMessage::FailedToMakeFile),
    }
}
//Function to change the \ to / and reverse.
pub fn adapt_path(the_path: &PathBuf, target_os: &str) -> Result<PathBuf, HelpMessage> {
    let path_string : &str = match the_path.to_str() {
        Some(x) => x,
        None => return Err(HelpMessage::FailedWorkingPath),
    };
    let mut modified_path : String = String::new();
    if target_os.to_lowercase() != consts::OS {
        if target_os.to_lowercase() == "windows" {
            modified_path = path_string.replace("/","\\");

        } else if target_os.to_lowercase() == "unix" {
            modified_path = path_string.replace("\\","/");
        }
    }
    Ok(PathBuf::from(modified_path))
    
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
