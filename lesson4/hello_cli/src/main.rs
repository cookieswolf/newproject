extern crate clap;

use clap::{App, SubCommand};
use std::env;
use std::path::Path;
use std::process;
use std::fs::File;
use std::io::{Read};
use std::fmt; 

#[derive(Debug)]
struct PrintInfo {
  curdir: String,
  curexe: String,
}
impl fmt::Display for PrintInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       write!(f, "current dir: {} \ncurrent exe: {}", self.curdir, self.curexe)
    }
}
/*
    subcommand: open   cargo run -- open -f ./src/main.rs   //open file
    subcommand: print  cargo run -- print -p                //print current path informatiion
*/
fn main() {
        let matches = App::new("Lesson4 Program")
                          .version("1.0")
                          .author("sun joey")
                          .about("substrate homework")
                          .subcommand(SubCommand::with_name("open") //定义子命令open用于打开一个文件并显示，当然文件不能太大
                                      .about("open file")
                                      .arg_from_usage("-f, --file=[FN] 'print file content'"))
                          .subcommand(SubCommand::with_name("print") //定义子命令print用于打印出当前运行的可执行文件的名字以及运行目录并实现了实现针对结果结构体PrintInfo的Display Trait
                                      .about("print current path information")
                                      .arg_from_usage("-p,  'print current path information'"))
                          .get_matches();
        
        /*
         Example:  cargo run -- open -f ./src/main.rs
         Result:
          args[0]=target/debug/hello_cli
          args[1]=open
          args[2]=-f
          args[3]=./src/main.rs
        */
        let mut count=0;
        for argument in env::args() {
            println!("args[{}]={}", count,argument);
            count+=1;
        }
        println!("#######################################"); 
        
        if let Some(matches) = matches.subcommand_matches("open") {
              if matches.is_present("file") {
                let file=matches.value_of("file").unwrap();
                println!("Printing filename={:?}",file);
                if Path::new(&file).exists() {
                  println!("File exist!!");
                  let mut f = File::open(file).expect("[Error] File not found.");
                  let mut data = String::new();
                  f.read_to_string(&mut data).expect("[Error] Unable to read the  file.");
                  println!("{}", data);
              }
              else {
                  eprintln!("[Error] No such file or directory.");
                  process::exit(1);
              }
            } else {
                println!("The function needs to be improved ...");
            }
        }
        if let Some(_matches) = matches.subcommand_matches("print") {
          
          let info=PrintInfo{
            curdir:env::current_dir().unwrap().display().to_string(),
            curexe:env::current_exe().unwrap().display().to_string()
          };
          println!("Display:\n{}", info);
          println!("Debug:\n{:?}", info);
          
        }
     
}