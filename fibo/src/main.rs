use clap::Parser;

/// Compute Fibonacci suite values
#[derive(Parser, Debug)]
#[command(long_about = None)]
struct Args {
    /// The maximal number to print the fibo value of
    value: u32,

    /// Print intermediate values
    #[arg(short, long)]
    verbose: bool,

    /// The minimum number to compute
    #[arg(short, long, default_value_t = 0)]
    min: u32,
}

fn fibo(n: u32) -> Option<u32> {
    let mut previous_sum = 0;
    let mut sum: Option<u32> = Some(1);
    for _ in 1..=n {
        let temp: u32 = previous_sum;
        sum?;
        previous_sum = sum.unwrap();
        sum = temp.checked_add(previous_sum);
    }
    sum
}

fn main() {
    let args = Args::parse();
    for i in args.min..=args.value {
        match fibo(i) {
            Some(result) => {
                if args.verbose || i == args.value {
                    println!("fibo({i}) = {result}")
                }
            }
            None => break,
        }
    }
}
