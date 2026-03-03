use std::{env, env::consts, fs::{File, read_dir, create_dir, create_dir_all}};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let args : Vec<String> = env::args().collect();

    //let Ok((target_os, source_file, destination_file)) = parse_args(&args) else { panic!("Err: Wrong usage, try again.")};
    let (target_os, execute, x_value, d_flag, m_flag, source_file, destination, list_files) = match parse_args_advanced(&args) {
        Ok(x) => x,
        Err(x) => match x {
            HelpMessage::MissingFlagValueO => panic!("Got wrong or missing argument to -o flag."),
            _ => panic!("Got another error. Check usage."),
        }
    };
    println!("{}, {}, {}, {}", d_flag, m_flag, source_file.display(), destination);

    if m_flag {
        match create_dir(PathBuf::from(destination)) {
            Ok(_) => println!("Made directory {}.", destination),
            Err(x) => eprintln!("Failed with following error: {}", x),
        }
        match file_handler(target_os, execute, list_files, destination, d_flag, m_flag) {
            Ok(x) => println!("Succesfully created files."), 
            Err(e) => panic!("Got an error: {:?}, Check usage.", e),
        }
    } else if d_flag {
        let new_file = File::create(destination)?;
        //1MB capacity for the buffer, TODO: make this an optional flag, which can be controlled.  
        let mut write_buf = BufWriter::with_capacity(1000000, new_file);
        let start_boiler: String = start_boilerplate(target_os, true, destination);
        write_buf.write(start_boiler.as_bytes());

        let grande_string : String = match d_flag_handler(target_os, execute, list_files, destination) {
            Ok(x) => x,
            Err(e) => panic!("Got an error: {:?}", e),
        };
        write_buf.write(grande_string.as_bytes());
        if execute {
            let execute_boiler : String = match executable_boiler(target_os, &x_value, destination, d_flag, m_flag) {
                Ok(x) => x,
                Err(e) => panic!("Got an error: {:?}",e),
            };
            write_buf.write(execute_boiler.as_bytes());
        }
        let end_boiler: String = end_boilerplate(target_os);
        write_buf.write(end_boiler.as_bytes());
        //TODO: Fix this for -d implementation.
        //if execute == true {
        //    let exec_string = match executable_boiler(target_os, &source_file) {
        //        Ok(x) => x,
        //        Err(x) => panic!("Got an error: {:?}", x),
        //    };
        //    write_buf.write(exec_string.as_bytes());
        //}
        //This pushes the contents of the buffer to the file. 
        write_buf.flush()?;

    } else {
        let new_file = File::create(destination)?;
        //1MB capacity for the buffer, TODO: make this an optional flag, which can be controlled.  
        let mut write_buf = BufWriter::with_capacity(1000000, new_file);
        let start_boiler: String = start_boilerplate(target_os, false, destination);
        write_buf.write(start_boiler.as_bytes());
        let file_content: String = make_file_boilerplate(target_os, &source_file, destination, false, false);
        write_buf.write(file_content.as_bytes());
        if execute {
            let execute_boiler : String = match executable_boiler(target_os, &x_value, destination, d_flag, m_flag) {
                Ok(x) => x, 
                Err(e) => panic!("Couldn't work with given x flag value, got error: {:?}", e),
            };
            write_buf.write(execute_boiler.as_bytes());
        }
        let end_boiler: String = end_boilerplate(target_os);
        write_buf.write(end_boiler.as_bytes());

    }
    Ok(())

}
fn d_flag_handler(target_os: &str, execute: bool, source_files: Vec<PathBuf>, destination: &str) -> Result<String, HelpMessage> {
    let mut grande_string : String = String::new();
    println!("list of paths: {:?}", &source_files);
    for current_path in source_files {
        if current_path.is_dir() {
            println!("current directory: {}", &current_path.display());
            let mut list_files : Vec<PathBuf> = Vec::new();
            //TODO add this whole section for getting the iterator as it's own function, as this is
            //now already used 3 times in the code. 
            let mut directory_iterator = match read_dir(&current_path) {
                Ok(x) => x,
                Err(e) => {println!("got error: {:?}", e); return Err(HelpMessage::DirectoryDoesNotExist)},
            };
            while true {
                let current_entry = directory_iterator.next();
                println!("ENTRY of DIR: {:?}", current_entry);
                match current_entry {
                    Some(x) => match x {
                        Ok(y) => list_files.push(y.path()),
                        Err(e) => {eprintln!("got error: {:?}", e); return Err(HelpMessage::FailedToGetFile)},
                    },
                    None => break,
                }
            }
            println!("value of list_files : {:?}", list_files);
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

//TODO: remove this code from the "main" function and just use this function, loop over it for when
//handed directory. Then test it.
fn file_handler(target_os: &str, execute: bool, source_files: Vec<PathBuf>, destination: &str, d_flag: bool, m_flag: bool) -> Result<(), HelpMessage> {

    for current_path in source_files {
        if current_path.is_dir() {
            let mut list_files : Vec<PathBuf> = Vec::new();
            let mut directory_iterator = match read_dir(&current_path) {
                Ok(x) => x,
                Err(e) => {println!("got error: {:?}", e); return Err(HelpMessage::DirectoryDoesNotExist)},
            };
            while true {
                let current_entry = directory_iterator.next();
                match current_entry {
                    Some(x) => match x {
                        Ok(y) => list_files.push(y.path()),
                        Err(e) => {eprintln!("got error: {:?}", e); return Err(HelpMessage::FailedToGetFile)},
                    },
                    None => break,
                }
            }
            file_handler(target_os, execute, list_files, destination, d_flag, m_flag);
        } else {

            let mut my_path : PathBuf = PathBuf::new();
            println!("current OS is: {}", consts::OS);
            if consts::OS == "windows" {
                my_path = PathBuf::from(format!("{}\\{}", destination, &current_path.display()));
            } else if consts::OS == "linux" || consts::OS == "macos" {
                my_path = PathBuf::from(format!("{}/{}", destination, &current_path.display()));
            }
            println!("PATH to create file is : {}", &my_path.display());
            //let new_file = match File::create(my_path) {
            //    Ok(x) => x,
            //    Err(e) => {eprintln!("got error: {}", e); return Err(HelpMessage::FailedToMakeFile)}
            //};
            let new_file = match file_nested_dirs(&my_path) {
                Ok(file) => file,
                Err(e) => {eprintln!("Got error message: {:?}", e); return Err(e)},
            };
            //1MB capacity for the buffer, TODO: make this an optional flag, which can be controlled.  
            let mut write_buf = BufWriter::with_capacity(1000000, new_file);

            let start_boiler: String = start_boilerplate(target_os, true, destination );
            write_buf.write(start_boiler.as_bytes());

            let file_content : String = make_file_boilerplate(target_os, &current_path, destination, false, true);
            write_buf.write(file_content.as_bytes());

            if execute {
                // Here I use &current path instead of x_value, I do this because on -m flag it makes no sense
                // to specify a file name for all of the new files to execute, so instead each file
                // will execute the file it creates. 
                let execute_boiler : String = match executable_boiler(target_os, &current_path, destination, d_flag, m_flag) {
                    Ok(x) => x,
                    Err(e) => panic!("Got an error: {:?}",e),
                };
                write_buf.write(execute_boiler.as_bytes());
            }

            let end_boiler: String = end_boilerplate(target_os);
            write_buf.write(end_boiler.as_bytes());

            //TODO: reimplement this, exectuable_boiler() needs its own target file when there are multiple
            //files in the directory.
            //if execute == true {
            //    let exec_string = match executable_boiler(target_os, &current_path) {
            //        Ok(x) => x,
            //        Err(x) => panic!("Got an error: {:?}", x),
            //    };
            //    write_buf.write(exec_string.as_bytes());
            //}
            //This pushes the contents of the buffer to the file. 
            match write_buf.flush() {
                Ok(x) => println!("Successfully wrote file."),
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
        Err(e) => Err(HelpMessage::FailedToMakeFile),
    }
}
//Function to change the \ to / and reverse.
fn adapt_path(the_path: &PathBuf, target_os: &str) -> Result<PathBuf, HelpMessage> {
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

fn executable_boiler(os_type: &str, source_file: &PathBuf, destination : &str, d_flag: bool, m_flag: bool) -> Result<String, HelpMessage> {
    let mut execute_string : String = String::new();

    let mod_dest : Vec<&str> = destination.split('.').collect();
    let no_suffix_dest : &str = mod_dest[0];

    let file_name : &str = match &source_file.file_name() {
        Some(x) => x.to_str().unwrap(),
        None => panic!("No file name. Unrecoverable error."),
    };
    let adapted_path : PathBuf = match adapt_path(&source_file, os_type) {
        Ok(x) => x,
        Err(e) => return Err(e),
    };
    let mut no_prefix_adapted_path : &str = "";
    if os_type.to_lowercase() == "windows" {
        no_prefix_adapted_path = match adapted_path.strip_prefix("\\") {
            Ok(x) => x.to_str().unwrap(),
            Err(e) => {eprintln!("Failed to strip prefix: {:?}", e); adapted_path.to_str().unwrap()},
        };

    } else if os_type.to_lowercase() == "unix" {
        no_prefix_adapted_path = match adapted_path.strip_prefix("/") {
            Ok(x) => x.to_str().unwrap(),
            Err(e) => {eprintln!("Failed to strip prefix: {:?}", e); adapted_path.to_str().unwrap()},
        };

    }

    if os_type.to_lowercase() == "windows" {
        execute_string.push_str(
                "DELAY 100\n\
                STRINGLN Set-ExecutionPolicy RemoteSigned -Scope CurrentUser\n\
                DELAY 200\n\
                ");

    } 
    if d_flag {
        if os_type.to_lowercase() == "windows" {
            execute_string.push_str(format!(
                    "STRINGLN $code = Get-Content .\\$HOME\\{}\\{} -Raw\n\
                    DELAY 400\n\
                    STRINGLN Invoke-Expression $code\n\
                    ", no_suffix_dest, source_file.display()).as_str());

        } else if os_type.to_lowercase() == "unix" {
            execute_string.push_str(format!(
                    "STRINGLN chmod +x $HOME/{}/{}\n\
                    DELAY 100\n\
                    STRINGLN $HOME/{}/{}\n\
                    ",no_suffix_dest, no_prefix_adapted_path, destination, no_prefix_adapted_path).as_str());

        } else {
            return Err(HelpMessage::WrongArgOS)
        }

    } else if m_flag {
        if os_type.to_lowercase() == "windows" {
            execute_string.push_str(format!(
                    "STRINGLN $code = Get-Content .\\$HOME\\{}\\{} -Raw\n\
                    DELAY 400\n\
                    STRINGLN Invoke-Expression $code\n\
                    ", no_suffix_dest, file_name).as_str());

        } else if os_type.to_lowercase() == "unix" {
            execute_string.push_str(format!(
                    "STRINGLN chmod +x $HOME/{}/{}\n\
                    DELAY 100\n\
                    STRINGLN $HOME/{}/{}\n\
                    ", no_suffix_dest, file_name, no_suffix_dest, file_name).as_str());

        } else {
            return Err(HelpMessage::WrongArgOS)
        }

    } else {
        if os_type.to_lowercase() == "windows" {
            execute_string.push_str(format!(
                    "STRINGLN $code = Get-Content .\\$HOME\\{} -Raw\n\
                    DELAY 400\n\
                    STRINGLN Invoke-Expression $code\n\
                    ",source_file.display()).as_str());

        } else if os_type.to_lowercase() == "unix" {
            execute_string.push_str(format!(
                    "STRINGLN chmod +x $HOME/{}\n\
                    DELAY 100\n\
                    STRINGLN $HOME/{}\n\
                    ", source_file.display(), source_file.display()).as_str());

        } else {
            return Err(HelpMessage::WrongArgOS)
        }

    }
    Ok(execute_string)
}

fn make_file_boilerplate(os_type: &str, source_file: &PathBuf, dest: &str, d_flag: bool, m_flag: bool) -> String {
    let mut mf_string: String = String::new();
    // Can only call .parent() on a PathBuf that is valid for the current OS (on which the command
    // runs). It doesn't work if i modify the path to the target OS first and then try to call
    // .parent() on it. That's why I do this before here, to create 2 seperate PathBuf's, one for
    // creating directories and the other for making the file itself.  
    let path_parent : PathBuf = match &source_file.parent() {
        Some(x) => PathBuf::from(x),
        None => panic!("Unrecoverable, failed to get parent path."),
    };
    let mod_path_parent :PathBuf = match adapt_path(&path_parent, os_type) {
        Ok(x) => x,
        Err(e) => panic!("Unrecoverable error processing changes to path. Got: {:?}", e),
    };   
    let mod_path : PathBuf = match adapt_path(&source_file, os_type) {
        Ok(x) => x,
        Err(e) => panic!("Unrecoverable error processing changes to path. Got: {:?}", e),
    };
    let file_name : &str = match &source_file.file_name() {
        Some(x) => x.to_str().unwrap(),
        None => panic!("No file name. Unrecoverable error."),
    };
    // To get rid of the file extensions as it looks weird to call a directory 'script.txt' for
    // example.
    let mod_dest : Vec<&str> = dest.split('.').collect();
    let no_suffix_dest : &str = mod_dest[0];

    if os_type.to_lowercase() == "windows" {
        if d_flag {
            mf_string.push_str(format!(
                "STRINGLN New-Item -ItemType Directory -Path \"$HOME\\{}\\{}\" -Force\n\
                " , no_suffix_dest, mod_path_parent.display()).as_str())
        }
        mf_string.push_str(
        "STRINGLN $file = @'\n\
        ");
    } else if os_type.to_lowercase() == "unix" {
        //TODO create_dir here essentially acts like d_flag check, since d and m can't happen at
        //the same time, chain these. Do check if they actually do the right thing.
        if d_flag {
            mf_string.push_str(format!(
                "STRINGLN mkdir -p $HOME/{}/{}\n\
                STRINGLN cat > $HOME/{}/{}\n\
                " ,no_suffix_dest, mod_path_parent.display(), 
                &no_suffix_dest, mod_path.display()
                ).as_str());

        } else if m_flag {
            mf_string.push_str(format!(
                "STRINGLN cat > $HOME/{}/{}\n\
                ", &no_suffix_dest, file_name).as_str());
        } else {
            //This should use regular 'dest' as it's the normal base case where the user doesn't do
            //multi file creation. 
            mf_string.push_str(format!(
                "STRINGLN cat > $HOME/{}\n\
                ", file_name).as_str());

        }
    }
    if let Ok(lines) = read_lines(&source_file) {
        //print_type_of(&lines);
        for line in lines.map_while(Result::ok) {
            let mut mod_line : String = line.clone();
            // Its necessary to check if the line is empty. The ducky script runs so fast that
            // especially on windows a STRINGLN with nothing after will be printed out as
            // "TRINGLN", without the "S" at the start, which powershell just loses for some
            // reason. this check is to mitigate that and try to write the lines correctly.
            match line.trim().is_empty() {
                true => {
                    mod_line.push_str("ENTER\n");
                    mf_string.push_str(mod_line.as_str());
                },
                false => {
                    //Changes to the line applied here before being written to the buffer.
                    mod_line.insert_str(0, "STRINGLN ");
                    mod_line.push_str("\n");
                    mf_string.push_str(mod_line.as_str());
                },
            }

        }
    } else {
        println!("Err: File you specified doesn't exist or something else went wrong. Your file: {}", &source_file.display());
        panic!("Stopped due to above error.")
    }
    if os_type.to_lowercase() == "windows" {
        if m_flag {
            mf_string.push_str(format!(
                "STRINGLN '@\n\
                STRINGLN Set-Content -Path $HOME\\{}\\{} -Value $file\n\
                ", &no_suffix_dest, file_name).as_str());

        } else if d_flag {
            mf_string.push_str(format!(
                "STRINGLN '@\n\
                STRINGLN Set-Content -Path $HOME\\{}\\{} -Value $file\n\
                ", &no_suffix_dest, mod_path.display()).as_str());
        } else {
            mf_string.push_str(format!(
                "STRINGLN '@\n\
                STRINGLN Set-Content -Path $HOME\\{} -Value $file\n\
                ", dest).as_str());

        }
    } else if os_type.to_lowercase() == "unix" {
        mf_string.push_str(
        "CTRL d\n\
        ");
    }
    mf_string
}

//TODO make this a return statement with Result<String, HelpMessage>
fn start_boilerplate(os_type: &str, is_dir : bool ,dest: &str) -> String {
    
    let mut os_start_string :String = String::new(); 
    let mod_dest : Vec<&str> = dest.split('.').collect();
    let no_suffix_dest : &str = mod_dest[0];

    if os_type.to_lowercase() == "windows" {
        os_start_string.push_str(
            "WINDOWS\n\
            DELAY 400\n\
            STRING powershell\n\
            DELAY 200\n\
            ENTER\n\
            DELAY 1000\n\
        ");
        if is_dir {
            os_start_string.push_str(format!(
            "STRINGLN New-Item -Path \"$HOME\\{}\" -Type Directory\n\
            ", no_suffix_dest).as_str());
        }
        
    } else if os_type.to_lowercase() == "unix" {
        os_start_string.push_str(
            "GUI\n\
            DELAY 400\n\
            STRING terminal\n\
            DELAY 200\n\
            ENTER\n\
            DELAY 400\n\
        ");
        if is_dir {
            os_start_string.push_str(format!(
                "STRINGLN mkdir $HOME/{}\n\
            ", no_suffix_dest).as_str());
        }
    } else {
        println!("Something went wrong the boilerplate start function got the wrong OS type: {}", os_type);
        //TODO: proper error handling.
        panic!();
    }

    os_start_string

}

//Add whatever the final part of the Bad usb script should do here.
fn end_boilerplate(os_type: &str ) -> String {
    let mut os_end_string: String = String::new();
    if os_type.to_lowercase() == "windows" {
        os_end_string.push_str(
            "STRING end of programn ! Do what you want now...\n");
        
    } else if os_type.to_lowercase() == "unix" {
        os_end_string.push_str(
            "STRING end of programn! Do other stuff here if you want now...\n");

    } else {
        println!("Something went wrong the boilerplate start function got the wrong OS type: {}", os_type);
        //TODO: proper error handling.
        panic!();
    }
    os_end_string
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

#[derive(Debug)]
enum HelpMessage {
    NotEnoughArgs,
    WrongArgOS,
    FileDoesNotExist,
    MissingFlagValueO,
    MissingFlagValueX,
    DirectoryDoesNotExist,
    FailedToGetFile,
    FailedToMakeFile,
    BufferFlushFailed,
    FailedToCreateDirectory,
    NoValueForFlagM,
    NoValueForFlagD,
    CannotCombineFlagsMF,
    NoDestinationSpecified,
    NoParentPath,
    FailedMakingDirs,
    FailedWorkingPath,
    FailedRecursionFS,
    FailedStrippingPrefix,
}

fn parse_args_advanced(args: &[String]) -> Result<(&str, bool, PathBuf, bool, bool, PathBuf, &str, Vec<PathBuf>), HelpMessage> {
    
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

    if args.len() < 3 {
        return Err(HelpMessage::NotEnoughArgs)
    }

    while true {
        let c_arg = iterator_args.next();
        match c_arg {
            Some(x) => 
            match x.as_str() {
                "-o" => match iterator_args.next() {
                            Some(x) => os_target = x,
                            None => return Err(HelpMessage::MissingFlagValueO)
                        },
                "-x" => {
                            executable = true;
                            match iterator_args.next() {
                                Some(x) => x_value = PathBuf::from(x),
                                None => return Err(HelpMessage::MissingFlagValueX)
                            }
                            
                        },
                //TODO: this has to be the last option given. Mention in help.
                "-d" => match iterator_args.next() {
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
                                while true {
                                    let current_entry = directory_iterator.next();
                                    match current_entry {
                                        Some(x) => match x {
                                            Ok(y) => list_files.push(y.path()),
                                            Err(e) => {eprintln!("got error: {:?}", e); return Err(HelpMessage::FailedToGetFile)},
                                        },
                                        None => break,
                                    }
                                }
                                return Ok((os_target, executable, x_value, d_flag, m_flag, source_path, target_dest, list_files))
                                
                            },
                            None => return Err(HelpMessage::NoValueForFlagD)
                },
                //TODO: this has to be the last option given. Mention in help.
                "-m" => match iterator_args.next() {
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
                                        while true {
                                            let current_entry = directory_iterator.next();
                                            match current_entry {
                                                Some(x) => match x {
                                                    Ok(y) => list_files.push(y.path()),
                                                    Err(e) => {eprintln!("got error: {:?}", e); return Err(HelpMessage::FailedToGetFile)},
                                                },
                                                None => break,
                                            }
                                        }
                                        return Ok((os_target, executable, x_value, d_flag, m_flag, source_path, target_dest, list_files))
                                        },
                            None => return Err(HelpMessage::NoValueForFlagM),
                        },
                s => {if count > 0 {target_dest = s} else {count += 1; source_path = PathBuf::from(s)}}, 
            },
            None => break,
        }
    }
    Ok((os_target, executable, x_value, d_flag, m_flag, source_path, target_dest, list_files))
}


//old function
fn parse_args(args: &[String]) -> Result<(&str, &str, &str), String> {
    
    println!("{:?}, ", args);
    match args.len() {
        // the first argument is automatic and is the binary path to where the script is running
        // from.
        0 | 1 | 2 => {
            Err(help_message("Need minimum 2 arguments, see above instructions."))
        },
        3 => {
            let src_file = &args[1];
            let dst_file = &args[2];


            return Ok(("windows", src_file, dst_file))
        },
        4 => {
            Err(help_message("3 arguments not possible, see above instructions."))
        },
        5 => {
            if &args[1] == "-o" {
                if &args[2].to_lowercase() != "windows" && &args[2].to_lowercase() != "unix" {
                    return Err(help_message("OS type for -o not recongnized. Only 'Unix' & 'Windows' are valid. Case insensitive.")) 
                }
            } else {
                return Err(help_message("Illegal arguments, check usage above."))
            }
            let os_type = &args[2];
            let src_file = &args[3];
            let dst_file = &args[4];
            Ok((os_type, src_file, dst_file))
        },
        _ => {
            Err(help_message("Too many arguments, see above instructions."))
        }  
    }
    
}

fn help_message(msg: &str) -> String {
    println!("USAGE: busb [source_file] [destination_file]");
    println!("Possible Flags:");
    println!("  -o Selects the OS type for the command to target. If you are running on linux use 'Unix'. Default: 'Windows'");
    println!("  -h Prints this help message.");
    println!("example:");
    println!("  busb my_test.sh my_new_file.txt");
    println!("example with OS selection:");
    println!("  busb -o Unix my_test.sh my_new_file.txt");
    return String::from(msg)
}
