use std::{path::Path, fs::File, io::{self, BufRead}, error::Error};

pub fn read_nodesl_file() -> Result<Vec<String>, Box<dyn Error>> {
    let path = Path::new("nodes.jsonl");

    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);
    let txt: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    Ok(txt)
}