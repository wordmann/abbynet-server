use std::fs::{File, OpenOptions};
use std::io::{Write, Read, ErrorKind, BufReader, BufRead};
use std::path::Path;
use std::{io, fs};
use std::net::{UdpSocket, SocketAddr, TcpListener};
use std::time::SystemTime;

const EXPTIME: u64 = 2630000;

fn main() {
    // Bind the server socket to the host and port
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind");


    println!("Server listening on 0.0.0.0:8080");

    // Accept incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Connected to client: {}", stream.peer_addr().unwrap());

                let mut buffer = [0; 1024];
                match stream.read(&mut buffer) {
                    Ok(_) => {
                        let received_data = String::from_utf8_lossy(&buffer);
                        println!("Received data: {}", received_data);

                        let inbound = received_data.to_string().trim().to_owned();
                        let mut response: String = " ".to_owned(); 

                        // filename&password&wumlcontent
                        let mk_check : Vec<&str>= inbound.split("&").collect();
                        if mk_check.len() > 2 {
                            let mut cur_time;
                            match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                                Ok(n) => cur_time = n.as_secs(),
                                Err(_) => panic!("SystemTime before UNIX EPOCH!"),
                            }

                            let pass: bool = check_timetable(cur_time, mk_check[0].to_owned(), mk_check[1].to_owned());

                            if pass {
                                pub_file(mk_check[2], mk_check[0].to_owned(), mk_check[1].to_owned());
                                response = "//---file probably published".to_string();
                            }else{
                                response = "//---wrong password".to_string();
                            }
                            

                        }else{

                        // STANDARD FILE FETCH
                        println!("Providing {} woml to {}", inbound, stream.peer_addr().unwrap());
                        response = get_woml(&inbound);
                            
                    }

                    if response == " " {
                        response = "//---wrong order".to_string();
                    }

                    stream.write_all(response.as_bytes()).expect("Write failed");
                        
                    }
                    Err(e) => {
                        eprintln!("Failed to read from connection: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e);
            }
        }
    }
}

fn get_woml(abm: &str) -> String {

    let abs_abm : Vec<&str>= abm.split("!").collect();
    let mut real_abm: String = abs_abm[0].to_owned();

    real_abm.push_str(".wuml");

    let filepath: String = real_abm;

    let contents : String = match fs::read_to_string(Path::new(&filepath)){
        Ok(file) => file,
        Err(_error) => "//---wrong ABM".to_owned(),
    };
    

    return contents;

}

fn check_timetable(time: u64, filename: String, pw: String) -> bool {

    let filepath: String = "meta.times".to_owned();

    let mut meta_file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(false)
        .open(filepath.clone()) {
            Ok(file) => file,
            Err(_err) => panic!("FUCK FUCK FUCK SHIT FUCK FUCK FUCK"),
        };

    let mut contents : String = "".to_owned(); 
    let _ = meta_file.read_to_string(&mut contents);


    

    let content_lines: Vec<&str> = contents.split("\n").collect();
    
    for line in content_lines {
        let filename_vec: Vec<&str> = line.split(" ").collect();
        
        if filename_vec[0] == filename {
            let expdate: u64 = match filename_vec[1].parse(){
                Ok(num) => num,
                Err(_err) => 1,
            };

            if expdate > time {
                println!("{}", filename_vec[2]);
                if filename_vec[2].trim_end() == pw {
                    return true;
                }else{
                    return false;
                }
            }else{
                replace_first_line(&filepath, &filename, "");
                return true;
            }
        }
    }

    return true;

}

fn pub_file(contents: &str, filename: String, pw: String){

    println!("Publishing {filename}!");

    let filepath: String = "meta.times".to_owned();

    let og_contents : String = match fs::read_to_string(Path::new(&filepath)){
        Ok(file) => file,
        Err(_error) => panic!("meta.times file not found."),
    };

    let mut new_metafile = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(false)
        .open(filepath.clone()) {
            Ok(file) => file,
            Err(_err) => panic!("FUCK FUCK FUCK SHIT FUCK FUCK FUCK"),
        };

    let cur_time: u64;
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => cur_time = n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }

    let exp_time = cur_time + EXPTIME;

    let meta_entry: String = format!("{filename} {exp_time} {pw}");
    let _ = replace_first_line(&filepath, &filename, &meta_entry);
    writeln!(new_metafile, "{}", og_contents);

    let mut wuml_filepath = filename;
    wuml_filepath.push_str(".wuml");
    
    let mut new_wumlfile = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(false)
        .open(wuml_filepath) {
            Ok(file) => file,
            Err(_err) => panic!("FUCK FUCK FUCK SHIT FUCK FUCK FUCK"),
        };

    let form_cont = format!("{}", contents);
    writeln!(new_wumlfile, "{}", form_cont);

}

fn replace_first_line(filename: &str, search_string: &str, replacement: &str) -> std::io::Result<()> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let mut lines = reader.lines();

    let first_line = lines
        .next()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Empty file"))??;

    if first_line.contains(search_string) {
        let mut output = File::create(filename)?;

        output.write_all(replacement.as_bytes())?;
        output.write_all(b"\n")?;

        for line in lines {
            output.write_all(line?.as_bytes())?;
            output.write_all(b"\n")?;
        }

        output.flush()?;
    }

    Ok(())
}