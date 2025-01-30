use std::path::PathBuf;
use std::fs::read_dir;
use std::mem::replace;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;
use std::fs::File;
use std::io::Read;

pub fn get_directory_contents(path: &PathBuf, dump: &mut Vec<PathBuf>) -> Result<usize, String> {
    let contents = match read_dir(path) {
        Ok(contents) => contents,
        Err(e) => return Err(format!("{e:?}"))
    };
    let _ = replace(dump, contents.filter_map(Result::ok).map(|x| x.path()).collect());
    Ok(dump.len())
}

pub struct SyntaxHighlighter {
    ps: SyntaxSet,
    ts: ThemeSet
}

pub struct SyntaxLine {
    pub text: Vec<String>,
    pub colour: Vec<(u8, u8, u8)>
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self { ps: SyntaxSet::load_defaults_newlines(), ts: ThemeSet::load_defaults() }
    }

    pub fn load_file(&mut self, path: &PathBuf) -> Vec<SyntaxLine> {
        let mut output: Vec<SyntaxLine> = vec![];
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(_) => return vec![]
        }

        let syntax: &SyntaxReference = self.ps.find_syntax_by_extension(
            match path.extension() {
                Some(ext) => ext.to_string_lossy().to_string(),
                None => String::from("txt")
            }.as_str()
        ).unwrap_or_else(|| self.ps.find_syntax_by_extension("txt").unwrap()); 

        let mut highlighter = HighlightLines::new(&syntax, &self.ts.themes["base16-ocean.dark"]);

        for line in LinesWithEndings::from(&contents) {
            let regions = highlighter.highlight_line(line, &self.ps).unwrap();
            let mut l: SyntaxLine = SyntaxLine { text: vec![], colour: vec![] };
            for region in &regions {
                l.text.push(region.1.to_string());
                l.colour.push((
                    region.0.foreground.r,
                    region.0.foreground.g,
                    region.0.foreground.b
                ))
            }
            output.push(l);
        }

        output
    }
}
