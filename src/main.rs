use json_diff_plus::{compare_json, compare_json_readable_output, compare_json_str_pretty_print};
use serde_json::json;

fn main() {
    let left = json!([{
        "a":{
            "b":{
                "c":333,
                "c1":111,
                "c2":[
                        {"d":1,"d1":2}
                    ]
            },
            "b1":55,
            "b2": [{"b2.a":[1,2],"b2.b":"2b2b"}],
        }
    }]);

    let right = json!([{
        "a":{
            "b":{
                "c":33,
                "c1":111,
                "c2":[
                        {"d":4,"d1":2}
                    ]
            },
            "b1":55,
            "b2": [{"b2.a":"abc","b2.c":[11]}],
            "b3": [23,34,11,22,33],
        }
    }]);

    compare_json_str_pretty_print(
        left.clone().to_string().as_str(),
        right.clone().to_string().as_str(),
    );
    println!("=============================================");

    let left = json!({
        "a":{"b":2},
        "a2": 3,
    });
    let right = json!({
        "a":2,
        "a2":{"b":3},
    });
    compare_json_str_pretty_print(
        left.clone().to_string().as_str(),
        right.clone().to_string().as_str(),
    );

    println!("=============================================");

    let left = json!([{
        "a":{"b":2},
    }]);
    let right = json!([{
        "a":{
                "b": 2,
                "b3":["b","n","l"]
            },
    }]);
    compare_json_str_pretty_print(
        left.clone().to_string().as_str(),
        right.clone().to_string().as_str(),
    );
}
