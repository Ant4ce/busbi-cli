use std::{env, fs::{File, create_dir}};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

mod boilerplates;
use boilerplates::{executable_boilerplate, start_boilerplate, end_boilerplate, make_file_boilerplate};
mod help_msg;
use help_msg::{HelpMessage};
mod arguments;
use arguments::{parse_args_advanced};
mod handlers;
use handlers::{file_handler, d_flag_handler};

fn main() -> io::Result<()> {
    let args : Vec<String> = env::args().collect();

    //let Ok((target_os, source_file, destination_file)) = parse_args(&args) else { panic!("Err: Wrong usage, try again.")};
    let (target_os, execute, x_value, d_flag, m_flag, source_file, destination, list_files, close_window) = match parse_args_advanced(&args) {
        Ok(x) => x,
        Err(e) => match e {
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
            let execute_boiler : String = match executable_boilerplate(target_os, &x_value, destination, d_flag, m_flag) {
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
            let execute_boiler : String = match executable_boilerplate(target_os, &x_value, destination, d_flag, m_flag) {
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





