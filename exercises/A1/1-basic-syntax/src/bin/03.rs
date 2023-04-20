fn main() {
    let input = [23, 82, 16, 45, 21, 94, 12, 34];

    // TODO
    let max = input.iter().max().unwrap();
    let min = input.iter().min().unwrap();
    println!("{max} is largest and {min} is smallest");
}
