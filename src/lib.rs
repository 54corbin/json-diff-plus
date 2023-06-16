use colored::Colorize;
use serde_json::{json, Value};
use std::{collections::HashSet, ops::Sub};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Discrepency {
    ///
    ///Represent a property that only exists in left side
    ///
    OnlyLeft { pointer: String, value: String },
    ///
    ///Represent a property that only exists in right side
    ///
    OnlyRight { pointer: String, value: String },
    ///
    ///Represent a property exists in both left and right side with different value
    ///
    Both {
        pointer: String,
        value_left: String,
        value_right: String,
    },
}

pub fn compare_json_str_pretty_print(left: &str, right: &str) {
    let discrepency = compare_json_str_readable_output(left, right);
    for ele in discrepency.lines() {
        if ele.contains("<-->") {
            println!("{}", ele.bright_yellow());
        } else {
            println!("{ele}");
        }
    }
}

pub fn compare_json_str_readable_output(left: &str, right: &str) -> String {
    let left: Value = serde_json::from_str(left).unwrap();
    let right: Value = serde_json::from_str(right).unwrap();
    compare_json_readable_output(left, right)
}

//merge right_only property into left side by looking up the first value that
//exists in left side then insert the right_only key-value paire into the  value
fn merge_right_only_to_left(
    pointer: &str,
    left: &mut serde_json::Value,
    right: &serde_json::Value,
) {
    let mut pointer = pointer;
    while let Some((a, b)) = pointer.rsplit_once('/') {
        pointer = a;
        if let Some(parent) = left.pointer_mut(a) {
            let rv = right.pointer(format!("{a}/{b}").as_str()).unwrap();

            match parent {
                Value::Object(p) => {
                    p.insert(b.to_string(), rv.to_owned());
                }
                Value::Array(arr) => {
                    arr.push(json!(format!("NAN <--> {:?}", rv)));
                }
                //Ideally, will never run into this arm
                p => *p = json!(format!("{p} <-[WARN]-> {:?}", rv)).into(),
            };
            break;
        }
    }
}

pub fn compare_json_readable_output(left: serde_json::Value, right: serde_json::Value) -> String {
    let mut left2 = left.clone();
    let right2 = right.clone();
    for diff in compare_json(left, right) {
        match diff {
            Discrepency::OnlyLeft { pointer, value } => {
                let value = format!("{value} <--> NAN");
                *left2.pointer_mut(&pointer).unwrap() = value.into();
            }
            Discrepency::OnlyRight { pointer, value: _ } => {
                merge_right_only_to_left(pointer.as_str(), &mut left2, &right2);
            }
            Discrepency::Both {
                pointer,
                value_left,
                value_right,
            } => {
                let value = format!("{value_left} <--> {value_right}");
                // *left2.pointer_mut(&pointer).unwrap() = value.into();
                match left2.pointer_mut(&pointer) {
                    Some(val) => {
                        *val = value.into();
                    }
                    None => {
                        //Ideally, should never run into this arm
                        println!("{}", serde_json::to_string_pretty(&left2).unwrap());
                        println!(
                            "[[[[[[[[[[[[Warning!! pointer={:#?};\t vL={:#?}; \t vR={:#?}]]]]]]]]]]]]",
                            pointer, value_left, value_right
                        );
                    }
                }
            }
        }
    }

    serde_json::to_string_pretty(&left2).unwrap()
}

pub fn compare_json_str(left: &str, right: &str) -> Vec<Discrepency> {
    let left: Value = serde_json::from_str(left).unwrap();
    let right: Value = serde_json::from_str(right).unwrap();

    self::compare_json(left, right)
}
///
///To compare tow json generate a vec that contains all the differences between left and right
///
pub fn compare_json(left: serde_json::Value, right: serde_json::Value) -> Vec<Discrepency> {
    let mut results: Vec<Discrepency> = Vec::<Discrepency>::new();

    let left_pointers = traverse_json(&left);
    let right_pointers = traverse_json(&right);

    // classify properties that only exists in left or only exists in right
    let mut common_pointer = left_pointers
        .intersection(&right_pointers)
        .map(|s| s.to_owned())
        .collect();
    let left_only_pointer = left_pointers.sub(&common_pointer);
    let right_only_pointer = right_pointers.sub(&common_pointer);

    //hadle the case: left is an Obj but right is a fundimental type. eg: /a/b={"c":123} vs /a/b=123
    let mut ignore_left: Vec<String> = vec![];
    for ro in right_only_pointer.iter() {
        for lo in left_only_pointer.iter() {
            if lo.starts_with(ro) {
                ignore_left.push(lo.to_owned());
                common_pointer.insert(ro.to_string());
            }
        }
    }

    //hadle the case: right is an Obj but left is a fundimental type. eg: /a/b=123 vs /a/b={"c":123}
    let mut ignore_right: Vec<String> = vec![];
    for lo in left_only_pointer.iter() {
        for ro in right_only_pointer.iter() {
            if ro.starts_with(lo) {
                ignore_right.push(ro.to_owned());
                common_pointer.insert(lo.to_string());
            }
        }
    }

    // generate Discrepencies for the property that only exists in right json
    for ptr in left_only_pointer {
        if ignore_left.contains(&ptr) {
            continue;
        }
        let ptr = format!("/{ptr}");
        let val = left.pointer(&ptr).unwrap_or(&json!(null)).to_string();
        results.push(Discrepency::OnlyLeft {
            pointer: ptr,
            value: val,
        })
    }

    // generate Discrepencies for the property that only exists in left json
    for ptr in right_only_pointer {
        let ptr = format!("/{ptr}");
        let val = right.pointer(&ptr).unwrap_or(&json!(null)).to_string();
        results.push(Discrepency::OnlyRight {
            pointer: ptr,
            value: val,
        })
    }

    //compare common property
    for ptr in common_pointer {
        let ptr = format!("/{ptr}");
        let lv = left.pointer(&ptr);
        let rv = right.pointer(&ptr);

        if lv.ne(&rv) {
            results.push(Discrepency::Both {
                pointer: ptr,
                value_left: lv.unwrap_or(&json!(null)).to_string(),
                value_right: rv.unwrap_or(&json!(null)).to_string(),
            });
        }
    }

    results.sort();
    results
}

///
///traverse a json and collect it's pointer
///A Pointer is a Unicode string with the reference tokens separated by `/`.
///For more information read [RFC6901](https://tools.ietf.org/html/rfc6901).
///
fn traverse_json(json: &Value) -> HashSet<String> {
    let mut pointer_set: HashSet<String> = HashSet::<String>::new();

    match json {
        Value::Object(m) => {
            for (k, v) in m {
                if v.is_object() || v.is_array() {
                    for p in traverse_json(v) {
                        let fp: String = format!("{k}/{p}");
                        pointer_set.insert(fp);
                    }
                } else {
                    pointer_set.insert(format!("{k}"));
                }
            }
        }
        Value::Array(a) => {
            for (i, v) in a.into_iter().enumerate() {
                if v.is_object() || v.is_array() {
                    for p in traverse_json(v) {
                        let fp: String = format!("{i}/{p}");
                        pointer_set.insert(fp);
                    }
                } else {
                    pointer_set.insert(format!("{i}"));
                }
            }
        }
        x => {
            println!("{}", x)
        }
    };

    pointer_set
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn nested_obj() {}
}
