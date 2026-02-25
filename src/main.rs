use std::{env, fs::{File, read_dir}};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let args : Vec<String> = env::args().collect();
    // The format for the arguments will be as follows: 
    // busb [source_file] [destination_file]
    // flags will include:
    // -h for help
    // -o for OS type, either Windows or Unix, Default: Windows

    //let target_os : &str = &args[1];
    //let target_file : &str = &args[2];
    //let desired_file_name : &str = &args[3];

    //let Ok((target_os, source_file, destination_file)) = parse_args(&args) else { panic!("Err: Wrong usage, try again.")};
    let (target_os, execute, is_directory, source_file, destination_file, list_files) = match parse_args_advanced(&args) {
        Ok(x) => x,
        Err(x) => match x {
            HelpMessage::MissingFlagO => panic!("Got wrong or missing argument to -o flag."),
            _ => panic!("Got another error. Check usage."),
        }
    };

    //println!("the OS target is {target_os}, for target file {source_file} which will be turned into txt file with name {destination_file}");
    let new_file = File::create(destination_file)?;
    //1MB capacity for the buffer, TODO: make this an optional flag, which can be controlled.  
    let mut write_buf = BufWriter::with_capacity(1000000, new_file);
    
    let start_boiler: String = start_boilerplate(target_os, &source_file);
    write_buf.write(start_boiler.as_bytes());


    if let Ok(lines) = read_lines(&source_file) {
        //print_type_of(&lines);
        for line in lines.map_while(Result::ok) {
            let mut mod_line : String = line.clone();
            //Changes to the line applied here before being written to the buffer.
            mod_line.insert_str(0, "STRINGLN ");
            mod_line.push_str("\n");
            write_buf.write(mod_line.as_bytes());

        }
    } else {
        println!("Err: File you specified doesn't exist or something else went wrong. Your file: {}", &source_file.display());
    }

    let end_boiler: String = end_boilerplate(target_os, &source_file);
    write_buf.write(end_boiler.as_bytes());

    if execute == true {
        let exec_string = match executable_boiler(target_os, &source_file) {
            Ok(x) => x,
            Err(x) => panic!("Got an error: {:?}", x),
        };
        write_buf.write(exec_string.as_bytes());
    }
    //This pushes the contents of the buffer to the file. 
    write_buf.flush()?;
    Ok(())

}

//TODO: remove this code from the "main" function and just use this function, loop over it for when
//handed directory. Then test it.
fn file_handler(target_os: &str, execute: bool, source_file: PathBuf) -> Result<(), HelpMessage> {
    //println!("the OS target is {target_os}, for target file {source_file} which will be turned into txt file with name {destination_file}");
    let new_file = match File::create(&source_file) {
        Ok(x) => x,
        Err(e) => {eprintln!("got error: {}", e); return Err(HelpMessage::FailedToMakeFile)}
    };
    //1MB capacity for the buffer, TODO: make this an optional flag, which can be controlled.  
    let mut write_buf = BufWriter::with_capacity(1000000, new_file);
    
    let start_boiler: String = start_boilerplate(target_os, &source_file);
    write_buf.write(start_boiler.as_bytes());


    if let Ok(lines) = read_lines(&source_file) {
        //print_type_of(&lines);
        for line in lines.map_while(Result::ok) {
            let mut mod_line : String = line.clone();
            //Changes to the line applied here before being written to the buffer.
            mod_line.insert_str(0, "STRINGLN ");
            mod_line.push_str("\n");
            write_buf.write(mod_line.as_bytes());

        }
    } else {
        println!("Err: File you specified doesn't exist or something else went wrong. Your file: {}", &source_file.display());
    }

    let end_boiler: String = end_boilerplate(target_os, &source_file);
    write_buf.write(end_boiler.as_bytes());

    if execute == true {
        let exec_string = match executable_boiler(target_os, &source_file) {
            Ok(x) => x,
            Err(x) => panic!("Got an error: {:?}", x),
        };
        write_buf.write(exec_string.as_bytes());
    }
    //This pushes the contents of the buffer to the file. 
    match write_buf.flush() {
        Ok(x) => println!("Successfully wrote file."),
        Err(e) => {eprintln!("got error: {}", e); return Err(HelpMessage::BufferFlushFailed)} 
    };
    Ok(())
    
}

fn executable_boiler(os_type: &str, source_file: &PathBuf) -> Result<String, HelpMessage> {
    let mut execute_string : String = String::new();

    //TODO improve this here. Currently we just take the source file path, but if the source file
    //is nested in a directory that directory is also stored in the 'source_file' argument we pass
    //here, I want to be able just use the name of the file to create it on the host system running
    //the ducky script. 
    //Essentially one of the problems is that this will write stuff like:
    //$code = Get-Content .\test_files/short_test.txt -Raw
    //But notice how this wouldn't work on windows since it uses forward slashes. 
    //Need to handle this by checking OS type of where the command is run to generate ducky script. 
    //
    //EXAMPLE STRING for executing script on powershell windows: 
    //"""
    //STRINGLN $code = Get-Content .\\{YOUR_SOURCE_FILE} -Raw\n\
    //DELAY 400\n\
    //STRINGLN Invoke-Expression $code\n\
    //"""
    if os_type.to_lowercase() == "windows" {
        execute_string.push_str(
        "DELAY 100\n\
        STRINGLN Set-ExecutionPolicy RemoteSigned -Scope CurrentUser\n\
        DELAY 200\n\
        STRINGLN $code = Get-Content .\\{YOUR_SOURCE_FILE} -Raw\n\
        DELAY 400\n\
        STRINGLN Invoke-Expression $code\n\
        ");

    } else if os_type.to_lowercase() == "unix" {
        execute_string.push_str(format!(
            "STRINGLN chmod +x {}\n\
            DELAY 100\n\
            STRINGLN ./{}\n\
            ", source_file.display(), source_file.display()).as_str());
        
    } else {
        return Err(HelpMessage::WrongArgOS)
    }
    Ok(execute_string)
}

//TODO make this a return statement with Result<String, HelpMessage>
fn start_boilerplate(os_type: &str, source_file: &PathBuf) -> String {
    
    let mut os_start_string :String = String::new(); 

    if os_type.to_lowercase() == "windows" {
        os_start_string.push_str(
            "WINDOWS\n\
            DELAY 400\n\
            STRING powershell\n\
            DELAY 200\n\
            ENTER\n\
            DELAY 1000\n\
            STRINGLN $content = @\"\n\
        ");
        
    } else if os_type.to_lowercase() == "unix" {
        os_start_string.push_str(format!(
            "GUI\n\
            DELAY 400\n\
            STRING terminal\n\
            DELAY 200\n\
            ENTER\n\
            DELAY 400\n\
            STRINGLN cat > {}\n\
        ", source_file.display()).as_str());
    } else {
        println!("Something went wrong the boilerplate start function got the wrong OS type: {}", os_type);
        //TODO: proper error handling.
        panic!();
    }

    os_start_string

}

fn end_boilerplate(os_type: &str, source_file : &PathBuf) -> String {
    let mut os_end_string: String = String::new();
    if os_type.to_lowercase() == "windows" {
        os_end_string.push_str(format!(
            "STRINGLN \"@\n\
            STRINGLN Set-Content -Path $HOME\\{} -Value $content\n\
        ", source_file.display()).as_str());
        
    } else if os_type.to_lowercase() == "unix" {
        os_end_string.push_str(
            "CTRL d\n\
            ");

    } else {
        println!("Something went wrong the boilerplate start function got the wrong OS type: {}", os_type);
        //TODO: proper error handling.
        panic!();
    }
    os_end_string
}

//TODO: add the ability to run this recursively if the user provides a directory with -d.
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
    MissingFlagO,
    DirectoryDoesNotExist,
    FailedToGetFile,
    FailedToMakeFile,
    BufferFlushFailed,
}

fn parse_args_advanced(args: &[String]) -> Result<(&str, bool, bool, PathBuf, &str, Vec<PathBuf>), HelpMessage> {
    
    let mut iterator_args = args.into_iter();
    let _ = iterator_args.next();

    //TODO: Set this to 'Windows' or 'Unix' to make it default.
    let mut os_target : &str = ""; 
    let mut executable : bool = false;
    let mut is_directory : bool = false;
    let mut list_files : Vec<PathBuf> = Vec::new();
    let mut count : i32 = 0;
    //let mut source_path: &str = "";
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
                            None => return Err(HelpMessage::MissingFlagO)
                        },
                "-x" => executable = true,
                //TODO: this has to be the last option given. Mention in help.
                "-d" => match iterator_args.next() {
                            Some(x) => {
                                is_directory = true;
                                source_path = PathBuf::from(x);
                                if source_path.is_dir() != true {
                                    eprintln!("Argument provided is not a directory: {}", source_path.display());
                                    return Err(HelpMessage::DirectoryDoesNotExist)
                                }
                                let mut directory_iterator = match read_dir(&source_path) {
                                    Ok(x) => x,
                                    Err(e) => {println!("got error: {:?}", e); return Err(HelpMessage::DirectoryDoesNotExist)},
                                };
                                while true {
                                    let current_entry = directory_iterator.next();
                                    match current_entry {
                                        Some(x) => match x {
                                            Ok(y) => list_files.push(y.path()),
                                            Err(e) => {eprintln!("got error: {}", e); return Err(HelpMessage::FailedToGetFile)},
                                        },
                                        None => break,
                                    }
                                }
                                return Ok((os_target, executable, is_directory, source_path, target_dest, list_files))
                                
                            },
                            None => return Err(HelpMessage::DirectoryDoesNotExist)
                }, 
                s => {if count > 0 {target_dest = s} else {count += 1; source_path = PathBuf::from(s)}}, 
            },
            None => break,
        }
    }
    Ok((os_target, executable, is_directory, source_path, target_dest, list_files))
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
