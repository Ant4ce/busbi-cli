use std::{env, env::consts, fs::{File, read_dir, create_dir, create_dir_all}};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let args : Vec<String> = env::args().collect();

    //let Ok((target_os, source_file, destination_file)) = parse_args(&args) else { panic!("Err: Wrong usage, try again.")};
    let (target_os, execute, x_value, d_flag, m_flag, source_file, destination, list_files, close_window) = match parse_args_advanced(&args) {
        Ok(x) => x,
        Err(e) => match e {
            HelpMessage::MissingFlagValueO => panic!("Got wrong or missing argument to -o flag."),
            HelpMessage::PrintingHelp => {println!("Hope that helped :)"); return Ok(())},
            _ => panic!("Got another error. Check usage. Got: {:?}", e),
        }
    };
    println!("{}, {}, {}, {}", d_flag, m_flag, source_file.display(), destination);

    if m_flag {
        match create_dir(PathBuf::from(destination)) {
            Ok(_) => println!("Made directory {}.", destination),
            Err(x) => eprintln!("Failed with following error: {}", x),
        }
        match file_handler(target_os, execute, list_files, destination, d_flag, m_flag, close_window) {
            Ok(_x) => println!("Succesfully created files."), 
            Err(e) => panic!("Got an error: {:?}, Check usage.", e),
        }
    } else if d_flag {
        let new_file = File::create(destination)?;
        //1MB capacity for the buffer, TODO: make this an optional flag, which can be controlled.  
        let mut write_buf = BufWriter::with_capacity(1000000, new_file);
        let start_boiler: String = start_boilerplate(target_os, true, destination);
        let _ = write_buf.write(start_boiler.as_bytes());

        let grande_string : String = match d_flag_handler(target_os, execute, list_files, destination) {
            Ok(x) => x,
            Err(e) => panic!("Got an error: {:?}", e),
        };
        let _ = write_buf.write(grande_string.as_bytes());
        if execute {
            let execute_boiler : String = match executable_boiler(target_os, &x_value, destination, d_flag, m_flag) {
                Ok(x) => x,
                Err(e) => panic!("Got an error: {:?}",e),
            };
            let _ = write_buf.write(execute_boiler.as_bytes());
        }
        let end_boiler: String = end_boilerplate(target_os, close_window);
        let _ = write_buf.write(end_boiler.as_bytes());
        //This pushes the contents of the buffer to the file. 
        write_buf.flush()?;

    } else {
        let new_file = File::create(destination)?;
        let mut write_buf = BufWriter::with_capacity(1000000, new_file);
        let start_boiler: String = start_boilerplate(target_os, false, destination);
        let _ = write_buf.write(start_boiler.as_bytes());
        let file_content: String = make_file_boilerplate(target_os, &source_file, destination, false, false);
        let _ = write_buf.write(file_content.as_bytes());
        if execute {
            let execute_boiler : String = match executable_boiler(target_os, &x_value, destination, d_flag, m_flag) {
                Ok(x) => x, 
                Err(e) => panic!("Couldn't work with given x flag value, got error: {:?}", e),
            };
            let _ = write_buf.write(execute_boiler.as_bytes());
        }
        let end_boiler: String = end_boilerplate(target_os, close_window);
        let _ = write_buf.write(end_boiler.as_bytes());

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
            loop {
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
fn file_handler(target_os: &str, execute: bool, source_files: Vec<PathBuf>, destination: &str, d_flag: bool, m_flag: bool, close_window : bool) -> Result<(), HelpMessage> {

    for current_path in source_files {
        if current_path.is_dir() {
            let mut list_files : Vec<PathBuf> = Vec::new();
            let mut directory_iterator = match read_dir(&current_path) {
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
                let execute_boiler : String = match executable_boiler(target_os, &current_path, destination, d_flag, m_flag) {
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
            Err(e) => {eprintln!("No prefix: {:?}, Continuing...", e); adapted_path.to_str().unwrap()},
        };

    } else if os_type.to_lowercase() == "unix" {
        no_prefix_adapted_path = match adapted_path.strip_prefix("/") {
            Ok(x) => x.to_str().unwrap(),
            Err(e) => {eprintln!("No prefix: {:?}, Continuing...", e); adapted_path.to_str().unwrap()},
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
                    ", no_suffix_dest, no_prefix_adapted_path).as_str());

        } else if os_type.to_lowercase() == "unix" {
            execute_string.push_str(format!(
                    "STRINGLN chmod +x $HOME/{}/{}\n\
                    DELAY 100\n\
                    STRINGLN $HOME/{}/{}\n\
                    ",no_suffix_dest, no_prefix_adapted_path, no_suffix_dest , no_prefix_adapted_path).as_str());

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

        //Note that even in the below standard case where we just generate 1 file, we still need to
        //specify the name of the file we want to execute as that is the name of the file that will
        //be added to the execution busbi script. 
    } else {
        if os_type.to_lowercase() == "windows" {
            execute_string.push_str(format!(
                    "STRINGLN $code = Get-Content $HOME\\{} -Raw\n\
                    DELAY 400\n\
                    STRINGLN Invoke-Expression $code\n\
                    ",file_name).as_str());

        } else if os_type.to_lowercase() == "unix" {
            execute_string.push_str(format!(
                    "STRINGLN chmod +x $HOME/{}\n\
                    DELAY 100\n\
                    STRINGLN $HOME/{}\n\
                    ", file_name, file_name).as_str());

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
fn end_boilerplate(os_type: &str , close_window : bool) -> String {
    let mut os_end_string: String = String::new();
    if close_window {
        os_end_string.push_str("STRINGLN exit\n");


    } else {
        if os_type.to_lowercase() == "windows" {
            os_end_string.push_str(
                "STRING end of programn ! Do what you want now...\n");

        } else if os_type.to_lowercase() == "unix" {
            os_end_string.push_str(
                "STRING end of programn! Do other stuff here if you want now...\n");

        } else {
            panic!("Something went wrong the boilerplate start function got the wrong OS type: {}", os_type);
        }
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
    MissingFlagValueO,
    MissingFlagValueX,
    DirectoryDoesNotExist,
    FailedToGetFile,
    FailedToMakeFile,
    BufferFlushFailed,
    NoValueForFlagM,
    NoValueForFlagD,
    CannotCombineFlagsMF,
    NoDestinationSpecified,
    NoParentPath,
    FailedMakingDirs,
    FailedWorkingPath,
    FailedRecursionFS,
    PrintingHelp,
}

fn parse_args_advanced(args: &[String]) -> Result<(&str, bool, PathBuf, bool, bool, PathBuf, &str, Vec<PathBuf>, bool), HelpMessage> {
    
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



fn help_message()  {

    println!("");
    println!("\t\t\t\x1b[4;32;40;1mWELCOME TO BUSBI!\x1b[0m\n");
    println!(" This Command Line Tool is for creating bad USB scripts AKA ducky scripts for copying");
    println!(" over targeted scripts or entire folders of them onto another machine through a ducky script");
    println!(" and possibly executing said scripts.");
    println!(" Note that the intended use is for shell scripts like '.sh' and '.ps1' but any text based");
    println!(" file works. So you can copy code or whole essays this way.\n");
    println!(" The files generated by busbi use the Flipper Zero BadUSB File format, see their webpage:\n");
    println!(" https://developer.flipper.net/flipperzero/doxygen/badusb_file_format.html");
    println!("");
    print!(" ");
    print!("\x1b[4;31mUSAGE:\x1b[0m");
    println!(" busbi [Options] [SOURCE FILE/DIR] [DESTINATION FILE/DIR]\n");
    print!(" ");
    println!("\x1b[4;35mPossible Options:\x1b[0m\n");
    println!("\t-o  --os         Selects the OS that you are targeting, i.e where your Bad USB\n\
              \t                 script will run. If you are running the busbi command on a\n\
              \t                 Windows or Unix system but the machine you want to target with the\n\
              \t                 generated script is a linux machine, then set this to 'unix'.\n");
    print!("\t                 ");
    println!("\x1b[4mDefault: Windows\x1b[0m\n");
    println!("\t-h  --help       Prints this help message.\n");
    println!("\t-d  --directory  Use this flag to target a directory, this flag MUST be followed by\n\
              \t                 the SOURCE and DESTINATION arguments. CANNOT be combined with '-m'.\n\
              \t                 The specified directory will be written in it's entirety to the\n\
              \t                 bad usb file, which will have the name of your DESTINATION\n\
              \t                 argument. DESTINATION MUST end with '.txt'.\n\
              \t                 When the bad USB script is used on the target system it will\n\
              \t                 replicate the folders and files in their entirety on the host\n\
              \t                 system under the $HOME\\DESTINATION directory or $HOME/DESTINATION\n\
              \t                 if on Unix.\n\
              \t                 See the Examples below for more help.\n
        ");
    println!("\t-m  --many       Use this flag to target a directory containing many files and/or\n\
              \t                 scripts. Each file inside the directory and it's sub-directories\n\
              \t                 will be turned into their own bad usb scripts under the DESTINATION\n\
              \t                 folder it creates. Note that the DESTINATION folder will be created\n\
              \t                 in the directory where you ran the busbi command.\n\
              \t                 This flag CANNOT be used together with '-d'. Just like with '-d' you\n\
              \t                 must specify the SOURCE DIR and DESTINATION right after. Essentially\n\
              \t                 it must be the last flag/option you use.\n\
              \t                 The created script -if run- will copy the contents of their target\n\
              \t                 file onto the host and will be put under the $HOME/DESTINATION\n\
              \t                 folder.");
    print!("\t                 ");
    print!("\x1b[4mNote:\x1b[0m");
    print!(" If used with '-x' option, this will cause all generated bad\n\
              \t                 USB scripts to execute the file they copy over onto the host.\n\n");
    println!("\t-c  --close      Closes the terminal/powershell window when the bad USB script\n\
              \t                 finishes.\n");
    println!("\t-x  --execute    Use this option to specify 1 file to execute at the end of the\n\
              \t                 bad usb script. Can be used with '-d' and '-m'.\n\
              \t                 If used with '-m' it will make every bad USB script generated\n\
              \t                 execute the file it copied over. Note that you should still\n\
              \t                 specify the target folder you put in the DESTINATION.\n\
              \t                 See examples below.\n\
              \t                 If used with '-d' you simply specify the file in the directory\n\
              \t                 you are targeting that should be executed at the end when all\n\
              \t                 files have been copied over.\n\
              \t                 See the examples below.");
    print!("\t                 ");
    print!("\x1b[4mNote:\x1b[0m");
    print!(" This can be used to run scripts that were not explicitely\n\
              \t                 copied over, as long as they are in the $HOME/DESTINAITON folder.\n");
    print!(" ");
    println!("\x1b[4;32mExamples:\x1b[0m");
    println!("");
    println!("\tAll examples will assume the existence of these files and folder:\n");
    println!("\t\tscripts_test/
\t\t├── hello.sh
\t\t├── nested_folder
\t\t│   └── count.sh
\t\t└── whoami.sh
");
    println!("\t\x1b[4mSimple Usage:\x1b[0m\n");
    println!("\t\tbusbi scripts_test/hello.sh new_busbi.txt\n");
    println!("\tWill create the following file:");
    println!("\t\t.
\t\t├── new_busbi.txt
\t\t└── scripts_test
\t\t    ├── hello.sh
\t\t    ├── nested_folder
\t\t    │   └── count.sh
\t\t    └── whoami.sh
");
    println!("\t\x1b[4mTarget Unix and Close the terminal/powershell at script end:\x1b[0m\n");
    println!("\t\tbusbi -o unix -c scripts_test/hello.sh new_busbi.txt\n");
    println!("\t\x1b[4mTarget Unix and execute target shell script:\x1b[0m\n");
    println!("\t\tbusbi -o unix -x scripts_test/hello.sh scripts_test/hello.sh new_busbi.txt\n");
    println!("\t\x1b[4mTarget Unix and target directory for copying:\x1b[0m\n");
    println!("\t\tbusbi -o unix -d scripts_test/ large_busbi.txt\n");
    println!("\tWill create the following file:");
    println!("\t\t.
\t\t├── large_busbi.txt
\t\t└── scripts_test
\t\t    ├── hello.sh
\t\t    ├── nested_folder
\t\t    │   └── count.sh
\t\t    └── whoami.sh
");
    println!("\t\x1b[4mTarget Unix and create many bad USB scripts:\x1b[0m\n");
    println!("\t\tbusbi -o unix -m scripts_test/ new_folder\n");
    println!("\tWill create the following folder with files:");
    println!("\t\t.
\t\t├── new_folder
\t\t│   └── scripts_test
\t\t│       ├── hello.txt
\t\t│       ├── nested_folder
\t\t│       │   └── count.txt
\t\t│       └── whoami.txt
\t\t└── scripts_test
\t\t    ├── hello.sh
\t\t    ├── nested_folder
\t\t    │   └── count.sh
\t\t    └── whoami.sh
");
    print!("\t\x1b[4mNote:\x1b[0m");
    println!(" That the command recreated the nested folder structure and turned every\n\
        \t      script into a bad USB file without replacing the originals.\n");
    println!("\t\x1b[4mCreate many bad USB scripts and each executes copied file:\x1b[0m\n");
    println!("\t\tbusbi -x scripts_test -m scripts_test/ new_folder\n");
    println!("\t\x1b[4mUnix/Close/Execute/Target directory:\x1b[0m\n");
    println!("\t\tbusbi -o unix -c -x scripts_test/nested_folder/count.sh -d scripts_test/ large_busbi.txt\n");
    println!("\tWill still just create a single file:");
    println!("\t\t.
\t\t├── large_busbi.txt
\t\t└── scripts_test
\t\t    ├── hello.sh
\t\t    ├── nested_folder
\t\t    │   └── count.sh
\t\t    └── whoami.sh
");
}
