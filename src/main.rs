use itertools::Itertools;
use std::time;
use std::env;
use rand::Rng;

fn main() {

    let args: Vec<String> = env::args().collect();
    let mut stack: Vec<i32> = vec![];
    let mut payout: Vec<i32> = vec![];

    //println!("args {:?}", args);

    if args.len() == 3 {
        let stack_str: Vec<&str> = args[1].split(',').collect();
        let payout_str: Vec<&str> = args[2].split(',').collect();

        //println!("stack_str: {:?}", stack_str);
        //println!("payout_str: {:?}", payout_str);

        for str in stack_str{
            stack.push( str.trim().parse::<i32>().unwrap_or_default() );
        }
        for str in payout_str{
            payout.push( str.trim().parse::<i32>().unwrap_or_default() );
        }

    }else{
        stack = vec![ 50, 30, 20, 5, 5, 4, 4, 4, 4, 4, 3 ];
        payout = vec![ 70, 30, 9,  8, 7, 6, 5, 4, 3, 2, 2 ];
        //let stack: Vec<i32> =      vec![21, 89, 90];
        //let mut payout: Vec<i32> = vec![50, 30, 20];
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

    println!("stack: {:?}",stack);
    println!("payout: {:?}",payout);

    //Tysen's SICM method
    let now = time::Instant::now();
    let _ = sicm(&stack, &mut payout, &mut payout_expected);
    println!("\npayout_expected: {:?}", payout_expected);
    println!("SICM method. done with {:?} msec.", now.elapsed().as_millis());

    
    //erase
    for p in payout_expected.iter_mut(){
        *p = 0.0;
    }
    
    //Malmuth-Harville method
    let now = time::Instant::now();
    let _ = icm(&stack, &mut payout, &mut payout_expected);
    println!("\npayout_expected: {:?}", payout_expected);
    println!("Malmuth-Harville method. done with {:?} msec.", now.elapsed().as_millis());
    

    return;

}


fn sicm(stack: &Vec<i32>, payout: &mut Vec<i32> , payout_expected: &mut Vec<f32>){
//SICM method or Tysen's method
//Two Plus Two Forums >> Poker Strategy >> Poker Theory & GTO
//New algorithm to calculate ICM for large tournaments
//https://forumserver.twoplustwo.com/15/poker-theory-amp-gto/new-algorithm-calculate-icm-large-tournaments-1098489/

    payout.sort();

    let trial_count = 1000000;
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
        sicm_trial(&stack_weight, payout, payout_expected, trial_count);
    }

    //println!("payout:{:?}, trial_count:{:?}, stack_total:{:?}, stack_weight{:?}, stack:{:?}, payout_expected:{:?}", 
    //        payout, trial_count, stack_total, stack_weight, stack, payout_expected);

}

fn sicm_trial(stack_weight: &Vec<f32>, payout: &mut Vec<i32>, payout_expected: &mut Vec<f32>, trial_count: i32){
    let mut rng = rand::thread_rng();

    //let mut scores = (0. as f32, 0 as i32);
    let mut scores_vec: Vec<(f32, i32)> = Vec::with_capacity(stack_weight.len());
    //let mut results : Vec<i32> = vec![0; stack_weight.len()];

    let mut i = 0;
    for w in stack_weight{
        scores_vec.push( ((rng.gen_range(0.0..1.0) as f32).powf(*w), i) );
        //dbg!!!!!!!!!
        //scores_vec.push((0.9_f32.powf(*w), i));
        i = i + 1;
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