#![feature(globs)]
use std::io::Command;
use std::io::process::ProcessOutput;
use std::io::process::ProcessExit;
use std::io::process::ProcessExit::*;
use std::os;

struct Options {
    gett_enable : bool,
    verbose     : bool,
    debug       : bool,
}

fn print_help() {
    let help = "gettit v0.0.1

This program fixes the uppercase and lowercase letters in ge.tt urls and others.

Usage:
gettit [OPTIONS] -l [link]

Options:
 -h: print this help screen
 -d: debug            (default: off)
 -v: verbose output   (default: off)
 -g: use ge.tt fixing (default: on)

 -l [link]: use this link
\n";
    println!("{}", help);
}

fn get_link(mut args: Vec<String>) -> Option<String> {
    for i in range(0u, args.len()){
        match args[i].as_slice() {
            "-l" => return Some(args[i+1].clone()),
            _    => continue,
        }
    }
    return None;
}

fn parse_args(mut args: &Vec<String>) -> Result<Options,&'static str> {
    let mut _args = box args;
    let default_options = Options { gett_enable : true,
                                    verbose     : false,
                                    debug       : false, };
    
    let mut options = default_options;
    
    match _args.len() {
        0 => { print_help();
               return Err("Help printed"); },
        _ => {
            for arg in _args.iter() {
                match arg.as_slice() {
                    "-h" => { print_help();
                              return Err("Help printed"); },
                    "-g" => options.gett_enable = true,
                    _ => continue,
                }
            }
        }
    }
    return Ok(options);
}

fn find_in_string(c: char, s: &String) -> Option<uint> {
    for (i, ch) in s.chars().enumerate() {
        if ch == c {
            return Some(i);
        }
    }
    return None;
}

struct LinkedChars {
    locations: Vec<uint>,
    letters: Vec<char>,
}

struct LinkedLink {
    link: String,
    list: LinkedChars,
}

type Mutations = Vec<LinkedChars>;

impl LinkedLink {
    fn new(link: String) -> LinkedLink {
        let letters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
            .to_string();

        let mut _loc: Vec<uint> = Vec::new();
        let mut _abc: Vec<char> = Vec::new();
        
        for (i, c) in link.chars()
            .skip(match find_in_string('/', &link)
                  { Some(i) => i + 1,
                    None    => panic!("Error: link failed to parse!"), })
            .enumerate() {
                match find_in_string(c, &letters)
                { Some(_) => { _loc.push(i);
                               _abc.push(c); },
                  None    => match c {
                      '/' => break,
                      _   => continue }
                }
            }
        
        return LinkedLink { link      : link,
                            list : LinkedChars { locations : _loc,
                                                 letters   : _abc, }, };
    }

    fn curl_mutations(&self, mutations: Mutations) {
        let offset = match find_in_string('/', &self.link) {
            Some(i) => i + 1,
            None => 0,
        };
        
        for mutation in mutations.iter() {
            let mut link = self.link.clone();
            let mut link_: Vec<u8> = Vec::new();
            link_.push_all(link.as_bytes());
           
            for (i, c) in mutation.letters.iter().enumerate() {
                link_[mutation.locations[i] + offset] = *c as u8;
            }
            
            link = match String::from_utf8(link_) {
                Ok(s) => s,
                Err(s)    => "".to_string() };
            spawn(proc() { println!("{}: {}", link.clone(), check_link(link))});
        }
    }
}

impl std::clone::Clone for LinkedChars {
    fn clone(&self) -> LinkedChars {
        return LinkedChars { letters: self.letters.clone(),
                             locations: self.locations.clone() };
    }

    fn clone_from(&mut self, source: &LinkedChars) {
        self.letters = source.letters.clone();
        self.locations = source.locations.clone();
    }
}


fn mutate(index: uint, string: LinkedChars) -> Mutations {
    let mut result: Mutations = Vec::new();
    if index >= string.letters.len() {
        result.push(string);
        return result;
    }

    let mut s = string.clone();
    s.letters[index] = s.letters[index].to_uppercase();

    result.push_all(mutate(index+1, s).as_slice());
    result.push_all(mutate(index+1, string).as_slice());

    return result;
}

fn curl(link: String) -> ProcessOutput {
    return match Command::new("curl").arg(link).output() {
        Ok(out) => out,
        Err(e)  => panic!("failed to execute: {}", e),
    };
}

fn check_link(link: String) -> bool {
    let mut string = "curl ".to_string();
    string.push_str(link.as_slice());
    string.push_str(" | grep \"Page not found\"");

    let output = match Command::new("sh").arg("-c").arg(string).output() {
        Ok(out) => out,
        Err(e)  => panic!("Failed to execute: {}", e),
    };

    match String::from_utf8(output.output) {
        Ok(s)  =>
        { match s.as_slice() {
            "" => return true,
            _  => return false, }}
        Err(e) => panic!("Error: {}", e),
    }
}

fn main() {
    let args = os::args();
    let opts = parse_args(&args);
    
    let mut link = LinkedLink::new(match get_link(args) {
        Some(l) => l,
        None    => panic!("Failed to parse link"), });
    
    println!("Link: {}\nLocations: {}\nLetters: {}",
             link.link, link.list.locations, link.list.letters);

    let mutations = mutate(0u, link.list.clone());

    link.curl_mutations(mutations);
}
