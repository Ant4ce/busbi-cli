use std::path::PathBuf;

use crate::help_msg::HelpMessage;
use crate::handlers::{adapt_path, read_lines};

pub fn executable_boilerplate(os_type: &str, source_file: &PathBuf, destination : &str, d_flag: bool, m_flag: bool) -> Result<String, HelpMessage> {
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
            Err(_e) => {eprintln!("No prefix: Continuing..."); adapted_path.to_str().unwrap()},
        };

    } else if os_type.to_lowercase() == "unix" {
        no_prefix_adapted_path = match adapted_path.strip_prefix("/") {
            Ok(x) => x.to_str().unwrap(),
            Err(_e) => {eprintln!("No prefix: Continuing..."); adapted_path.to_str().unwrap()},
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
                    "STRINGLN $code = Get-Content $HOME\\{}\\{} -Raw\n\
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
                    "STRINGLN $code = Get-Content $HOME\\{}\\{} -Raw\n\
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

pub fn make_file_boilerplate(os_type: &str, source_file: &PathBuf, dest: &str, d_flag: bool, m_flag: bool) -> String {
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
    // To get rid of the file extensions as it looks weird to call a directory 'script.txt'.
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

pub fn start_boilerplate(os_type: &str, is_dir : bool ,dest: &str) -> String {
    
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
    } 
    os_start_string

}

//Whatever String this function produces will be added to the end of any file produced.
//Use it as you see fit. Currently only runs if '-x' flag is NOT true.
//TODO make this more soffisticated with '-x' so both can be true at the same time. 
pub fn end_boilerplate(os_type: &str , close_window : bool) -> String {
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

