use std::fs::File;
use std::io::{self, BufRead};

const MAX_VEC_LENGTH :usize = 10;


#[derive(Debug, PartialEq, Eq)]
enum Equality {
    Equal,
    QueryGreater,
    QueryLesser,
}

pub fn file_to_array(file_path: &str) -> Vec<String> {
    let file = File::open(file_path).unwrap();
    let reader = io::BufReader::new(file);

    let mut result :Vec<String> = vec![];
    for line in reader.lines() {
        let a = &line.unwrap();
        result.push(a.to_string().to_lowercase());
    }
    result.sort();

    return result;
}
//Returns a vec with the elements matching the query. OBS, v must be lowercase and sorted.
pub fn query( query_in :&String, v :&Vec<String>) -> Vec<String> {
    let query = query_in.trim().to_lowercase();
    let mut at = v.len()/2;
    let mut upper = v.len();
    let mut lower = 0;
    loop {
        match is_equal(&query, &v[at]) {
            Equality::Equal => {
                let start_index = go_back(&v, &query, at);
                let end_index = go_forward(start_index, &v, &query, at);
                if end_index-start_index > MAX_VEC_LENGTH {
                    return v[start_index..start_index+MAX_VEC_LENGTH].to_vec();
                }
                return v[start_index..end_index].to_vec();
            }
            Equality::QueryLesser => {
                upper = at;
                at = lower + ((upper - lower)/2);
            }
            Equality::QueryGreater => {
                lower = at;
                at = lower + ((upper - lower)/2);

            }
        }
    }
}
fn go_back(v :&Vec<String>, query :&String, mut index :usize) -> usize {
    while index > 0 {
        index-=1;
        if is_equal(&query, &v[index]) != Equality::Equal {
            index+=1;
            break;
        }
    }
    return index;
}
fn go_forward(start :usize, v :&Vec<String>, query :&String, mut index :usize) -> usize {
    while index-start < MAX_VEC_LENGTH && index <= v.len() {
        index+=1;
        if is_equal(&query, &v[index]) != Equality::Equal {
            break;
        }
    }
    return index;
}
fn is_equal(query_in :&String, res :&String) -> Equality{
    let mut query = query_in.clone();
    if query.len() > res.len() {
        query = query[0..res.len()].to_string();
    }
    let q_chars = query.chars();
    let mut r_chars = res.chars();

    for c in q_chars {
        let to = r_chars.nth(0).unwrap();
        if c == to  {
            continue;
        } else if (c as u8) > (to as u8) {
            return Equality::QueryGreater;
        }
        return Equality::QueryLesser
    }
    return Equality::Equal;
}
