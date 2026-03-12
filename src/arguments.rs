use std::fs::read_dir;
use std::path::PathBuf;

use crate::help_msg::{HelpMessage, help_message};

pub fn parse_args_advanced(args: &[String]) -> Result<(&str, bool, PathBuf, bool, bool, PathBuf, &str, Vec<PathBuf>, bool), HelpMessage> {
    
    let mut iterator_args = args.into_iter();
    let _ = iterator_args.next();

    //TODO: Set this to 'Windows' or 'Unix' to make it default.
    let mut os_target : &str = "windows"; 
    let mut executable : bool = false;
    let mut x_value : PathBuf = PathBuf::new();
    let mut d_flag: bool = false;
    let mut m_flag : bool = false;
    let mut list_files : Vec<PathBuf> = Vec::new();
    let mut count : i32 = 0;
    let mut source_path: std::path::PathBuf = PathBuf::new();
    let mut target_dest: &str = "";
    let mut close_window : bool = false;


    loop {
        let c_arg = iterator_args.next();
        match c_arg {
            Some(x) => 
            match x.as_str() {
                "-o" | "--os"=> match iterator_args.next() {
                            Some(x) => os_target = x,
                            None => return Err(HelpMessage::MissingFlagValueO)
                        },
                "-x" | "--execute" => {
                            executable = true;
                            match iterator_args.next() {
                                Some(x) => x_value = PathBuf::from(x),
                                None => return Err(HelpMessage::MissingFlagValueX)
                            }
                            
                        },
                //TODO: this has to be the last option given. Mention in help.
                "-d" | "--directory" => match iterator_args.next() {
                            Some(x) => {
                                d_flag = true;
                                if m_flag == true {
                                    return Err(HelpMessage::CannotCombineFlagsMF)
                                }
                                source_path = PathBuf::from(x);
                                //TODO: This might be redundant, check given read_dir throwing error
                                //also, check. 
                                if source_path.is_dir() != true {
                                    eprintln!("Argument provided is not a directory: {}", source_path.display());
                                    return Err(HelpMessage::DirectoryDoesNotExist)
                                }
                                target_dest = match iterator_args.next() {
                                    Some(x) => x,
                                    None => return Err(HelpMessage::NoDestinationSpecified)
                                };
                                let mut directory_iterator = match read_dir(&source_path) {
                                    Ok(x) => x,
                                    Err(e) => {println!("got error: {:?}", e); return Err(HelpMessage::DirectoryDoesNotExist)},
                                };
                                loop {
                                    let current_entry = directory_iterator.next();
                                    match current_entry {
                                        Some(x) => match x {
                                            Ok(y) => list_files.push(y.path()),
                                            Err(e) => {eprintln!("got error: {:?}", e); return Err(HelpMessage::FailedToGetFile)},
                                        },
                                        None => break,
                                    }
                                }
                                return Ok((os_target, executable, x_value, d_flag, m_flag, source_path, target_dest, list_files, close_window))
                                
                            },
                            None => return Err(HelpMessage::NoValueForFlagD)
                },
                //TODO: this has to be the last option given. Mention in help.
                "-m" | "--many" => match iterator_args.next() {
                            Some(x) => {
                                        m_flag = true;
                                        if d_flag == true {
                                            return Err(HelpMessage::CannotCombineFlagsMF)
                                        }
                                        source_path = PathBuf::from(x);
                                        let mut directory_iterator = match read_dir(&source_path) {
                                            Ok(x) => x,
                                            Err(e) => {println!("got error: {:?}", e); return Err(HelpMessage::DirectoryDoesNotExist)},
                                        };
                                        target_dest = match iterator_args.next() {
                                            Some(x) => x,
                                            None => return Err(HelpMessage::NoDestinationSpecified)
                                        };
                                        loop {
                                            let current_entry = directory_iterator.next();
                                            match current_entry {
                                                Some(x) => match x {
                                                    Ok(y) => list_files.push(y.path()),
                                                    Err(e) => {eprintln!("got error: {:?}", e); return Err(HelpMessage::FailedToGetFile)},
                                                },
                                                None => break,
                                            }
                                        }
                                        return Ok((os_target, executable, x_value, d_flag, m_flag, source_path, target_dest, list_files, close_window))
                                        },
                            None => return Err(HelpMessage::NoValueForFlagM),
                        },
                "-c" | "--close" => close_window = true, 
                "-h" | "--help" => {
                                        help_message();
                                        return Err(HelpMessage::PrintingHelp)
                                    },
                s => {if count > 0 {target_dest = s} else {count += 1; source_path = PathBuf::from(s)}}, 
            },
            None => break,
        }
    }
    if args.len() < 3 {
        return Err(HelpMessage::NotEnoughArgs)
    }
    Ok((os_target, executable, x_value, d_flag, m_flag, source_path, target_dest, list_files, close_window))
}
