use std::vec::Vec;
use std::fs::File;
use std::io::Read;
use std::collections::HashSet;
use std::collections::HashMap;
use std::cmp::Ordering;

#[macro_use] extern crate rouille;
pub extern crate url;

extern crate regex;
use regex::Regex;

fn is_descriptor(c : char) -> bool
{
    c as u32 >= 0x2FF0 && c as u32 <= 0x2FFB
}
fn is_ascii(c : char) -> bool
{
    c as u32 <= 0xFF
}

type Mapping = HashMap<char, HashSet<char>>;

fn full_width_digits(text : &String) -> String
{
    let mut ret = text.clone();
    ret = ret.replace("1", "１");
    ret = ret.replace("2", "２");
    ret = ret.replace("3", "３");
    ret = ret.replace("4", "４");
    ret = ret.replace("5", "５");
    ret = ret.replace("6", "６");
    ret = ret.replace("7", "７");
    ret = ret.replace("8", "８");
    ret = ret.replace("9", "９");
    ret = ret.replace("0", "０");
    ret
}

struct ServerData {
    template: String,
    char_to_comp: Mapping,
    comp_to_char: Mapping,
    char_to_first_comp: HashMap<char, Vec<char>>,
    kanjidicplus: HashSet<char>,
    media: HashSet<char>,
    joyoplus: HashSet<char>,
    char_to_strokes: HashMap<char, u64>
}

impl ServerData {
    fn default(&self) -> String
    {
        let keys = vec!(
            ("input", ""),
            ("reverse_checked", ""),
            ("simple_checked", ""),
            ("selected_joyoplus", ""),
            ("selected_media", "selected"),
            ("selected_kanjidicplus", ""),
            ("selected_all", ""),
            ("output", ""),
        );
        let mut output = self.template.clone();
        for key in keys
        {
            output = output.replace(("{{{".to_string()+key.0+"}}}").as_str(), key.1);
        }
        
        output
    }
    fn search(&self, args : HashMap<&str, &str>) -> String
    {
        let mut output = self.template.clone();
        
        let reverse = args.contains_key("reverse");
        let simple = args.contains_key("simple");
        let filter_level =
        if let Some(x) = args.get("filter_level")
        {
            match *x
            {
                "joyoplus" => 0,
                "media" => 1,
                "kanjidicplus" => 2,
                "all" => 3,
                _ => 1
            }
        }
        else
        {
            3
        };
        
        let mut input = args.get("input");
        if let None = input
        {
            input = args.get("lookup");
        }
        
        
        if let Some(input) = input
        {
            output = output.replace("{{{input}}}", input);
            
            let lookup_output = 
            if reverse
            {
                ServerData::chars_to_components
            }
            else
            {
                ServerData::components_to_chars
            }
            (self, &mut HashSet::<char>::new(), &input.chars().collect::<HashSet<char>>(), !simple);
            
            let filter_function = |c|
            if filter_level == 0
                { self.joyoplus.contains(&c) }
            else if filter_level == 1
                { self.media.contains(&c) }
            else if filter_level == 2
                { self.kanjidicplus.contains(&c) }
            else
                { true }
            ;
            
            let mut lookup_vec = lookup_output.into_iter().filter(|c| if !reverse { filter_function(*c) } else { true } ).collect::<Vec<char>>();
            
            lookup_vec.sort();
            /*
            lookup_vec.sort_by(|A, B|
            {
                let a = self.get_strokes(*A);
                let b = self.get_strokes(*B);
                if a == b { Ordering::Equal }
                else if a == 0 || a > b { Ordering::Greater }
                else if b == 0 || a < b { Ordering::Less }
                else { panic!("logic in sorting is broken") }
            });
            */
            
            let mut stroke_mapping = HashMap::<u64, Vec<char>>::new();
            let mut stroke_counts = Vec::<u64>::new();
            for c in lookup_vec
            {
                let count = self.get_strokes(c);
                if !stroke_mapping.contains_key(&count)
                {
                    stroke_mapping.insert(count, Vec::<char>::new());
                    stroke_counts.push(count);
                }
                stroke_mapping.get_mut(&count).unwrap().push(c);
            }
            stroke_counts.sort();
            let mut output_list_html = "<span class=resetspacing>Ordered by stroke count:</span><br>".to_string();
            if stroke_counts.len() > 0
            {
                for count in stroke_counts
                {
                    output_list_html += &full_width_digits(&format!("\n{}：", count));
                    for c in stroke_mapping.get(&count).unwrap()
                    {
                        output_list_html += &"<c>";
                        output_list_html.push(*c);
                        output_list_html += &"</c>";
                        //output_list_html += &"<span style=\"-webkit-user-select:none;-moz-user-select:none;-ms-user-select:none;user-select:none;\">\u{200C}</span>"; // make selection double-taps or tap-and-holds single-character
                    }
                    output_list_html += &"<br>";
                }
            }
            else
            {
                output_list_html += &"(no matches)<br>";
            }
            output_list_html += &"<span class=resetspacing>Some stroke counts might be estimates.<br>Some characters might be too obscure for your system to render. Consider installing Hanazono Mincho (both HanaMinA and HanaMinB) if this is the case.</span>";
            
            output = output.replace("{{{output}}}", &output_list_html);
        }
        else
        {
            output = output.replace("{{{input}}}", "");
            output = output.replace("{{{output}}}", "");
        }
        
        output = output.replace("{{{reverse_checked}}}", if reverse {"checked"} else {""});
        output = output.replace("{{{simple_checked}}}", if simple {"checked"} else {""});
        
        output = output.replace("{{{selected_joyoplus}}}", if filter_level == 0 {"selected"} else {""});
        output = output.replace("{{{selected_media}}}", if filter_level == 1 {"selected"} else {""});
        output = output.replace("{{{selected_kanjidicplus}}}", if filter_level == 2 {"selected"} else {""});
        output = output.replace("{{{selected_all}}}", if filter_level == 3 {"selected"} else {""});
        
        output
    }
    
    fn chars_to_components(&self, seen : &mut HashSet<char>, set : &HashSet<char>, isrecursive : bool) -> HashSet<char>
    {
        let mut new = HashSet::<char>::new();
        
        for c in set
        {
            if let Some(comps) = self.char_to_comp.get(&c)
            {
                new = new.union(&comps).cloned().collect();
            }
        }
        
        new = new.difference(&seen).cloned().collect();
        for c in &new
        {
            seen.insert(*c);
        }
        
        if isrecursive && new.len() > 0
        {
            new = new.union(&self.chars_to_components(seen, &new, isrecursive)).cloned().collect();
        }
        
        new
    }
    
    fn component_to_chars(&self, seen : &mut HashSet<char>, in_c : char, isrecursive : bool) -> HashSet<char>
    {
        let mut new = HashSet::<char>::new();
        
        if let Some(chars) = self.comp_to_char.get(&in_c)
        {
            new = chars.clone();
        }
        
        new = new.difference(&seen).cloned().collect();
        for c in &new
        {
            seen.insert(*c);
        }
        
        if isrecursive && new.len() > 0
        {
            for c in new.clone()
            {
                for c2 in self.component_to_chars(seen, c, isrecursive)
                {
                    new.insert(c2);
                }
            }
        }
        
        new
    }
    
    fn components_to_chars(&self, seen : &mut HashSet<char>, set : &HashSet<char>, isrecursive : bool) -> HashSet<char>
    {
        let mut new = HashSet::<char>::new();
        let mut first = true;
        
        for c in set
        {
            if first
            {
                new = self.component_to_chars(&mut seen.clone(), *c, isrecursive);
            }
            else
            {
                new = new.intersection(&self.component_to_chars(&mut seen.clone(), *c, isrecursive)).cloned().collect();
            }
            first = false;
        }
        
        new
    }
    
    // TODO: interpret ② etc.
    fn get_strokes(&self, c : char) -> u64
    {
        if let Some(stroke_count) = self.char_to_strokes.get(&c)
        {
            *stroke_count
        }
        else if let Some(set) = self.char_to_first_comp.get(&c)
        {
            set.iter().map(|c2| self.get_strokes(*c2)).sum()
        }
        else
        {
            match c
            {
                '①' => 1,
                '②' => 2,
                '③' => 3,
                '④' => 4,
                '⑤' => 5,
                '⑥' => 6,
                '⑦' => 7,
                '⑧' => 8,
                '⑨' => 9,
                '⑩' => 10,
                '⑪' => 11,
                '⑫' => 12,
                '⑬' => 13,
                '⑭' => 14,
                '⑮' => 15,
                '⑯' => 16,
                '⑰' => 17,
                '⑱' => 18,
                '⑲' => 19,
                '⑳' => 20,
                
                '艹' => 4,
                '⺆' => 2,
                '既' => 11,
                '者' => 9,
                '𭕄' => 3,
                '卑' => 8,
                'サ' => 3,
                'コ' => 2,
                '㇇' => 1,
                '⺄' => 1,
                'よ' => 2,
                '⺌' => 3,
                '⺊' => 2,
                'い' => 2,
                'ス' => 2,
                'り' => 2,
                'ユ' => 2,
                '㇌' => 1, // should be 2 but is only actually used for composition in a single character where it has one stroke
                '㇉' => 1,
                '㇀' => 1,
                '㇓' => 1,
                '𛂦' => 2,
                '𮍌' => 5,
                '𭣔' => 5,
                '𮠕' => 8,
                '𬺻' => 5,
                _ => {println!("character {} has no stroke count", c); 0}
            }
        }
    }
}

fn ids_lines_to_mappings(lines : &Vec<String>) -> (Mapping, Mapping, HashMap<char, Vec<char>>)
{
    let mut char_to_comp = Mapping::new();
    let mut comp_to_char = Mapping::new();
    let mut char_to_first_comp = HashMap::<char, Vec<char>>::new();
    for line in lines
    {
        let mut tokens : Vec<String> = line.split("\t").map(|x| x.to_string()).collect();
        if tokens.len() < 3
        {
            continue;
        }
        tokens.remove(0);
        let kanji_chars : Vec<char> = tokens.remove(0).chars().collect();
        assert!(kanji_chars.len() == 1);
        let kanji = kanji_chars[0];
        let mut first = true;
        for token in tokens
        {
            for c in token.chars()
            {
                if c == kanji || is_descriptor(c) || is_ascii(c)
                {
                    continue;
                }
                
                if !char_to_comp.contains_key(&kanji)
                {
                    char_to_comp.insert(kanji, HashSet::<char>::new());
                }
                char_to_comp.get_mut(&kanji).unwrap().insert(c);
                
                if first && !char_to_first_comp.contains_key(&kanji)
                {
                    char_to_first_comp.insert(kanji, Vec::<char>::new());
                }
                char_to_first_comp.get_mut(&kanji).unwrap().push(c);
                
                if !comp_to_char.contains_key(&c)
                {
                    comp_to_char.insert(c, HashSet::<char>::new());
                }
                comp_to_char.get_mut(&c).unwrap().insert(kanji);
            }
            first = false;
        }
    }
    
    (char_to_comp, comp_to_char, char_to_first_comp)
}

fn build_stroke_count_mapping(lines : &Vec<String>) -> (HashMap<char, u64>)
{
    let mut stroke_counts = HashMap::<char, u64>::new();
    for line in lines
    {
        let mut tokens : Vec<String> = line.split("\t").map(|x| x.to_string()).collect();
        if tokens.len() < 3
        {
            continue;
        }
        let kanji_codepoint = tokens.remove(0);
        let info_type = tokens.remove(0); // kTotalStrokes
        if info_type != "kTotalStrokes"
        {
            continue
        }
        assert!(kanji_codepoint.starts_with("U+"));
        assert!(kanji_codepoint.len() == 6 || kanji_codepoint.len() == 7); // U+XXXX or U+XXXXX format
        //println!("`{}`", &kanji_codepoint[2..]);
        let kanji = std::char::from_u32(u32::from_str_radix(&kanji_codepoint[2..], 16).unwrap()).unwrap();
        let stroke_count_text = tokens.remove(0).split(" ").map(|x| x.to_string()).next().unwrap();
        let stroke_count = stroke_count_text.parse::<u64>().unwrap();
        stroke_counts.insert(kanji, stroke_count);
    }
    
    stroke_counts
}

fn load_to_string(fname : &str) -> std::io::Result<String>
{
    let mut file = File::open(fname)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    return Ok(string);
}

fn init() -> std::io::Result<ServerData>
{
    let template = load_to_string("template.html")?;
    let ids = load_to_string("ids.txt")?; // https://github.com/cjkvi/cjkvi-ids/blob/master/ids.txt
    let kanjidicplus_kanji = load_to_string("kanjidic2_kanji_plus.txt")?;
    let media_kanji = load_to_string("common.txt")?;
    let joyoplus_kanji = load_to_string("joyoplus2.txt")?;
    let unihan_dict_data = load_to_string("Unihan_DictionaryLikeData.txt")?;
    
    let ids_lines : Vec<String> = ids.lines().map(|x| x.to_string()).collect();
    let (char_to_comp, comp_to_char, char_to_first_comp) = ids_lines_to_mappings(&ids_lines);
    let stroke_lines : Vec<String> = unihan_dict_data.lines().filter(|x| x.starts_with("U+")).map(|x| x.to_string()).collect();
    
    Ok(ServerData { template, char_to_comp, comp_to_char, char_to_first_comp, kanjidicplus : kanjidicplus_kanji.chars().collect(), media : media_kanji.chars().collect(), joyoplus : joyoplus_kanji.chars().collect(), char_to_strokes : build_stroke_count_mapping(&stroke_lines) })
}

fn main() -> Result<(), std::io::Error>
{
    let serverdata = init()?;
    
    let mut kanji_asdf = serverdata.char_to_comp.keys().cloned().collect::<Vec<char>>();
    kanji_asdf.sort_unstable();
    let mut strokes_to_char = HashMap::<u64, Vec<char>>::new();
    for kanji in kanji_asdf
    {
        let strokes = serverdata.get_strokes(kanji);
        if !strokes_to_char.contains_key(&strokes)
        {
            strokes_to_char.insert(strokes, Vec::new());
        }
        strokes_to_char.get_mut(&strokes).unwrap().push(kanji);
    }
    
    let mut strokes_asdf = strokes_to_char.keys().cloned().collect::<Vec<u64>>();
    strokes_asdf.sort_unstable();
    println!("most strokes: {}", strokes_asdf.last().unwrap());
    println!("kanji with that many strokes:");
    for kanji in strokes_to_char.get(strokes_asdf.last().unwrap()).unwrap()
    {
        println!("{}", kanji);
    }
    
    
    println!("finished loading");
    
    assert!(!serverdata.kanjidicplus.contains(&'䏊'));
    
    let re = Regex::new(r"([?&;])([^=&;#]+)(=([^&;#]+))?").unwrap();
    
    let args = std::env::args().collect::<Vec<String>>();
    let address =
    if args.len() <= 1
    {
        "localhost:8000"
    }
    else
    {
        args.get(1).unwrap()
    };
    
    rouille::start_server(address, move |request|
    {
        let mystr = "&".to_string()+&url::percent_encoding::percent_decode(request.raw_query_string().as_bytes()).decode_utf8_lossy().into_owned();
        let matches = re.find_iter(mystr.as_str());
        
        let mut args = HashMap::<&str, &str>::new();
        for mymatch in matches
        {
            let split = mymatch.as_str().splitn(2, "=").collect::<Vec<_>>();
            if split.len() == 1
            {
                args.insert(&split[0][1..], "");
            }
            else
            {
                args.insert(&split[0][1..], split[1]);
            }
        }
        router!(request,
            (GET) (/) =>
            {
                rouille::Response::html(serverdata.default())
            },
            (GET) (/find) =>
            {
                
                rouille::Response::html(serverdata.search(args.clone()))
            },
            (GET) (/search) =>
            {
                
                rouille::Response::html(serverdata.search(args))
            },
            _ => rouille::Response::empty_404()
        )
    });
    
    Ok(())
}