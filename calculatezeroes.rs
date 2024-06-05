// Read input from out.txt and calculate the number of zeroes in the file do now load the file in memory because it can be very large
// Output the number of zeroes in the file

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

fn main() {
    let path = Path::new("out.txt");
    let file = File::open(&path).unwrap();
    let reader = io::BufReader::new(file);
    let mut count = 0;
    // let mut var = 4000003;
    // while(var > 0) {
    //     println!("{}", var);
    //     var=var/2;
    // }
    // define a f128 variable
    let mut f128_var = 11e4931;
    //for line in reader.lines() {
    //    let line = line.unwrap();
    //    count += line.matches('0').count();
    //}
    println!("{}", count);
}
//4933
// 4882
// seq 4e4931 4e4931