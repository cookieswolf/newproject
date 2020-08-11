use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time;
use std::env;
use std::num::ParseIntError;
use std::process;
//线程的处理函数
fn handle_client(mut stream: TcpStream) -> Result<(), Error>{
    //初始化缓冲区以及大小
    let mut buf = [0; 512];
    //因为缓冲区设置了大小，所以读取网络来的数据的时候不一定一次性的读完，所以这里设置了1000次循环。
    //其实也可以不这样做。
    for _ in 0..1000 {
        //读取stream里面的数据到buf中，并返回字节长度
        let bytes_read = stream.read(&mut buf)?;
        //如果字节长度为0，说明已经读完，返回退出
        if bytes_read == 0 {
            return Ok(());
        }
        //回写数据到客户端
        stream.write(&buf[..bytes_read])?;
        //防止过于频繁，休眠1秒钟
        thread::sleep(time::Duration::from_secs(1 as u64));
    }
     
    Ok(())
}
//验证字符串是否为数字
fn handle_number(number_str:&str)->Result<i32,ParseIntError>{
    match number_str.parse::<i32>(){
            Ok(n)=>Ok(n),
            Err(err)=>Err(err),
        }
 }

 //运行的格式为cargo run  8080  其中8080为端口的形式
fn main() -> std::io::Result<()> {
    //获命令行参数列表
    let args: Vec<String> = env::args().collect();
    //固定IP地址为本地
    let _ip = "127.0.0.1";
    //获取端口
    let mut _port = &args[1];
    //验证端口是否为数字，如果不是直接退出
    match handle_number(_port){
        Ok(n)=>println!("Port is number.{}",n),
        Err(err)=>{
            println!("Port must be number.Error: {:?}",err);
            process::exit(1);
        },
    }
    //拼接成127.0.0.1:8080的形式
    let conn = format!("{}:{}",  String::from(_ip), String::from(_port));   
    //在8080端口开始侦听
    let listener = TcpListener::bind(conn)?;
    //声明一个tcp线程连接池,用于保存连接
    let mut thread_vec: Vec<thread::JoinHandle<()>> = Vec::new();

    //循环侦听，直到有数据连接
    for stream in listener.incoming() {
        let stream = stream.expect("failed!");
        //开启一个线程
        let handle = thread::spawn(move || {
            handle_client(stream).unwrap_or_else(|error| eprintln!("{:?}", error));
        });
        //保存线程到连接池 
        thread_vec.push(handle);
    }

    //循环连接池，直到所有的线程完成任务再退出
    for handle in thread_vec {
        handle.join().unwrap();
    }

    Ok(())
}