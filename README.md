# Busbi-CLI tool

## Purpose

This project is for turning your scripts into bad USB scripts. Said scripts will copy over the contents of your shell or powershell 
script over onto a target machine and do other actions such as executing them if you so choose. 

You can also use this tool to copy over any files you wish to have portable with you through a bad USB script for your flipper zero 
such as config files or setup scripts. As long as the files are text based it will work.

Note: It creates BadUSB scripts based on the Flipper Zero standard described here:
https://developer.flipper.net/flipperzero/doxygen/badusb_file_format.html

## Usage

For creating a simple bad usb file out of a script, simply invoke busbi with the file you want to copy over to another system and the
name you want to give to the bad usb script. Make sure your bad usb script name ends with '.txt'

busbi [Options] [Target File] [New Bad USB script Name]

Example: 

`busbi my_script.sh my_badusb.txt`

this will create your script called "my_badusb.txt" in your current directory. Put it on your flipper zero and run it as a bad USB 
and it will create your original `my_script.sh` under the `$HOME/my_badusb/` directory.

Use `-x` option if you want the script to be executed at the end of the bad USB file, like so:

`busbi -x my_script.sh my_script.sh my_badusb.txt`

If you are running busbi on a linux machine but want to target a Windows system, use -o (for OS) like so:

`busbi -o Windows my_script.sh my_badusb.txt`

The `-o` flag accepts: 'Unix' & 'Windows'
Default: 'Windows'

You can combine these flags if you want:

`busbi -o windows -x my_script.sh my_script.sh my_badusb.txt`

## For more complex usage:

You can package a whole directory into a single bad USB script file the `-d` flag or `--directory` as the long version. 
You can use this as follows:

`busbi -d my_folder/ my_large_badusb.txt`

This will create the `my_large_badusb.txt` which will recreate the entire directory you gave it onto the target machine where
you run the bad USB script. The files and folders it creates will be under the `$HOME/my_large_badusb/` directory.

Combine this with `-x` and `-o` to explicitely target an OS and execute a specific file. 

`busbi -o unix -x my_folder/my_shell_script.sh -d my_folder/ my_bad_USB.txt`

This will recreate the entire directory (`my_folder`) and all it contains under `$HOME/my_bad_USB/` and then it will run the 
script at `$HOME/my_bad_USB/my_folder/my_shell_script.sh`.
