use itertools::Itertools;
use std::time;
//use std::env;
use rand::{Rng,SeedableRng};
use rand::rngs::SmallRng;
use clap::Parser;

#[derive(Parser)]
struct Args{
    /// player stack. comma-separated. ex. -s "2000,1500,1500,1000,500"
    #[arg(short, long, default_value = "50, 30, 20, 5, 5, 4, 4, 4, 4, 4, 3")]
    stack: String,

    /// payout structure. comma-separated. ex. -p "1000,700,500,370,270,200,140,90,84,80,80"
    #[arg(short, long, default_value = "70, 30, 9,  8, 7, 6, 5, 4, 3, 2, 2")]
    payout: String,

    /// optional. Default 1000000. sampling count for monte-carlo simulations. positive-integer.
    #[arg(short, long, default_value = "1000000")]
    count: u32,

    /// optional. use "smallrng" for random number generator. Default "thread_rng"
    #[arg(short, long)]
    x: bool,

    /// verbose output.
    #[arg(short, long)]
    verbose: bool,
}

fn main() {

    let args = Args::parse();

    let mut stack: Vec<i32> = vec![];
    let mut payout: Vec<i32> = vec![];

    let stack_str: Vec<&str> = args.stack.split(',').collect();
    let payout_str: Vec<&str> = args.payout.split(',').collect();

    for str in stack_str{
        stack.push( str.trim().parse::<i32>().unwrap_or_default() );
    }
    for str in payout_str{
        payout.push( str.trim().parse::<i32>().unwrap_or_default() );
    }


    let mut payout_expected: Vec<f32> = vec![0.0; stack.len()];

    if stack.len() > payout.len(){
        for _ in 0..(stack.len()-payout.len()) {
            payout.push(0);
        }
    }

    if payout.len() > stack.len(){
        for _ in 0..(payout.len()-stack.len()) {
            payout.pop();
        }
    }


    if args.verbose {
        println!("stack: {:?}",stack);
        println!("payout: {:?}",payout);
    }

    //Tysen's SICM method
    let now = time::Instant::now();
    let _ = sicm(&stack, &mut payout, &mut payout_expected, args.count, args.x);
    
    if args.verbose {
        println!("\npayout_expected: {:?}", payout_expected);
        println!("SICM method. done with {:?} msec.", now.elapsed().as_millis());
        
        if args.x {
            println!("rng: smallrng");
        }else {
            println!("rng: thread_rng");
        }

    }else {
        println!("{:?}", payout_expected);
    }

    
    /*
    //erase
    for p in payout_expected.iter_mut(){
        *p = 0.0;
    }
    
    //Malmuth-Harville method
    let now = time::Instant::now();
    let _ = icm(&stack, &mut payout, &mut payout_expected);
    println!("\npayout_expected: {:?}", payout_expected);
    println!("Malmuth-Harville method. done with {:?} msec.", now.elapsed().as_millis());
    */

    return;

}


fn sicm(stack: &Vec<i32>, payout: &mut Vec<i32> , payout_expected: &mut Vec<f32>, count: u32, smallrng_flg: bool){
//SICM method or Tysen's method
//Two Plus Two Forums >> Poker Strategy >> Poker Theory & GTO
//New algorithm to calculate ICM for large tournaments
//https://forumserver.twoplustwo.com/15/poker-theory-amp-gto/new-algorithm-calculate-icm-large-tournaments-1098489/

    payout.sort();

    let trial_count = count;
    //let mut stack_avg: f32 = 0.0;
    let mut stack_total = 0;
    let mut stack_weight: Vec<f32> = Vec::with_capacity(stack.len());

    for s in stack{
        stack_total = stack_total + *s;
    }

    //stack_avg = stack_total as f32 / stack.len() as f32;

    for s in stack{
        //stack_weight.push(stack_avg / *s as f32);
        stack_weight.push(stack_total as f32 / stack.len() as f32 / *s as f32);
    }

    for _ in 0..trial_count{
        sicm_trial(&stack_weight, payout, payout_expected, trial_count, smallrng_flg);
    }

    //println!("payout:{:?}, trial_count:{:?}, stack_total:{:?}, stack_weight{:?}, stack:{:?}, payout_expected:{:?}", 
    //        payout, trial_count, stack_total, stack_weight, stack, payout_expected);

}

fn sicm_trial(stack_weight: &Vec<f32>, payout: &mut Vec<i32>, payout_expected: &mut Vec<f32>, trial_count: u32, smallrng_flg: bool){
    
    let mut smallrng = SmallRng::from_entropy();
    let mut threadrng = rand::thread_rng();

    //let mut scores = (0. as f32, 0 as i32);
    let mut scores_vec: Vec<(f32, i32)> = Vec::with_capacity(stack_weight.len());
    //let mut results : Vec<i32> = vec![0; stack_weight.len()];

    let mut i = 0;
    if smallrng_flg {
        for w in stack_weight{
            scores_vec.push( ((smallrng.gen_range(0.0..1.0) as f32).powf(*w), i) );
            i = i + 1;
        }
    }else{
        for w in stack_weight{
            scores_vec.push( ((threadrng.gen_range(0.0..1.0) as f32).powf(*w), i) );
            i = i + 1;
        }
    }

    //println!("scores_vec {:?}", scores_vec);
    scores_vec.sort_by(|a, b| (a.0).partial_cmp(&(b.0)).unwrap());
    //println!("scores_vec sorted {:?}", scores_vec);

    for (payout, score) in payout.iter().zip(scores_vec.iter()){
        //println!("score.1 {:?}", score.1);
        //results[score.1 as usize] = *payout;
        payout_expected[score.1 as usize] = payout_expected[score.1 as usize] + *payout as f32 / trial_count as f32;
    }

    //println!("sicm_trial results: {:?}",results);


}


fn icm(stack: &Vec<i32>, payout: &mut Vec<i32> , payout_expected: &mut Vec<f32>){
//Malmuth-Harville method
//https://en.wikipedia.org/wiki/Independent_Chip_Model

    let mut stack_total = 0;

    payout.sort();
    payout.reverse();

    for s in stack{
        stack_total = stack_total + *s;
    }

    //println!("\n\n\n\n\n\n\n\n\n\n");

    for players in (0..(stack.len())).into_iter().permutations(stack.len()){
        //println!("\nPlayers {:?} ", players);

        let mut i = 0;
        let mut probabilities: f32 = 0.0;

        for p in &players{

            if i == 0 {

                //println!("Player{:?} {:?}th place stack:{:?}", *p, i, stack[*p]);
                probabilities = stack[*p] as f32;

            }else if i == players.len()-1 {

                //println!("probabilities: {:?}", probabilities as f32 / stack_total as f32);

                let mut k = 0;
                for p in &players{
                    //println!("player{:?}, {:?}th place", *p, k);
                    //println!("{:?}th place payout {:?}\n", k, payout[k]);

                    payout_expected[*p] = payout_expected[*p] + payout[k] as f32 * probabilities as f32 / stack_total as f32;

                    k = k + 1;
                }
                //println!( "payout_expected:{:?}", payout_expected );
                break;

            }else{

                //println!("Player{:?} {:?}th place stack:{:?}", *p, i, stack[*p]);

                let mut sum = 0;
                let mut j = 0;
                //println!( "i:{:?} j:{:?}", i, j );
                //println!( "players:{:?}", players );
                for p in &players{
                    sum = sum + stack[*p];
                    //println!( "i:{:?} j:{:?} sum:{:?} stack[j]:{:?}", i, j, sum, stack[j] );
                    j = j + 1;
                    if j == i { break; }
                }
                //println!("{:?} - {:?} stack sum: {:?}", 0, i-1, sum);
                probabilities = probabilities * stack[*p] as f32 / ( stack_total as f32 - sum as f32 );

            }
            i = i + 1;

        }

    }    

}