/* https://projecteuler.net/problem=843
This problem involves an iterative procedure that begins with a circle of n > 2 integers.
At each step every number is simultaneously replaced with the absolute difference of its two neighbours.
For any initial values, the procedure eventually becomes periodic.

Let S(N) be the sum of all possible periods for 3 <= n <= N .
For example, S(6) = 6 , because the possible periods for 3 <= n <= 6 are 1, 2, 3.

You are also given S(30)=20381.

Find S(100).
*/

use rand::Rng;
use std::cmp;
use std::cmp::Ordering;
use std::mem;
const K: usize = 10; // The number of stacks
const STACK_SIZE: usize = 1000;
use rayon::prelude::*;
use std::collections::HashSet;
use std::time::Instant;

//We use an array for the circle object for n integers;  An array if fixed in size at compile time.  So we have to use
//an array of 100 integers for the smaller circles.  A Vec implementation is more than 3 times slower.
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub struct Circle([u8; 100], usize);

impl Default for Circle {
    fn default() -> Self {
        Self([0; 100], 0)
    }
}

//Helper function for the multi_stack algorithm.  It implements a lexiconographic ordering for arrays
fn compare_arrays<T: Ord>(a: &[T], b: &[T]) -> Ordering {
    let min_length = a.len().min(b.len());

    for i in 0..min_length {
        match a[i].cmp(&b[i]) {
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
            Ordering::Equal => continue,
        }
    }

    a.len().cmp(&b.len())
}

//Helper function for the next() method on the array of integers.  It swaps two values in an array;  this is necessary
//because we use a rotation on the full array for the procedure outlined in the problem.
fn swap<T>(x: &mut [T], i: usize, j: usize) {
    let (lo, hi) = match i.cmp(&j) {
        Ordering::Equal => return, // No swapping necessary
        Ordering::Less => (i, j),
        Ordering::Greater => (j, i),
    };

    let (init, tail) = x.split_at_mut(hi);
    mem::swap(&mut init[lo], &mut tail[0]);
}

impl Circle {
    pub fn new(size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut l = [0; 100];
        for i in 0..size {
            l[i] = rng.gen_range(0..u8::MAX);
        }
        Self(l, size)
    }
}

// next() produces the iterative steps in the problem statement.  It returns the next Circle;  the circle object is not changed.
impl Circle {
    pub fn next(&mut self) -> Self {
        let len = self.1;
        let mut s1 = self.0.clone();
        let mut s2 = self.0.clone();
        s1.rotate_left(1);

        s2.rotate_right(1);
        if len < s1.len() {
            swap(&mut s1, len - 1, 99);
            swap(&mut s2, 0, len);
        }

        let mut l = [0; 100];

        for i in 0..len {
            let x = s1[i];
            let y = s2[i];
            l[i] = cmp::max(x, y) - cmp::min(x, y);
        }
        Self(l, self.1)
    }

    //helper function for the multi stack algoritm.  It returns the sum of the first 16 elements in the circle.
    pub fn sum(&self) -> u128 {
        return self.0[0..16].iter().map(|x| *x as u128).sum();
    }

    //getter for the array inside Circle
    pub fn get_array(&self) -> &[u8] {
        return &self.0;
    }
}

//Impementation of https://www.gabrielnivasch.org/fun/cycle-detection, an algorithm for cycle detection.
#[derive(Default, Copy, Clone, Debug)]
pub struct Stack(Circle, usize);
pub fn multi_stack(x0: Circle) -> Option<usize> {
    let mut stack: [[Stack; STACK_SIZE]; K] = [[Stack::default(); STACK_SIZE]; K];
    let mut h = [0 as usize; K]; // The stack sizes
    let mut time = 0;

    let mut x = x0;
    let mut k: usize;
    let mut j = 0;

    loop {
        k = (x.sum() % K as u128) as usize;
        for i in (0..h[k]).rev() {
            j = i;

            if !(compare_arrays(x.get_array(), stack[k][i].0.get_array()) == Ordering::Greater) {
                break;
            }
        }

        if j > 0 && stack[k][j].0 == x {
            return Some(time - stack[k][j].1);
        }
        h[k] = j + 2;
        stack[k][j + 1].0 = x;
        stack[k][j + 1].1 = time;
        x = x.next();

        time += 1;
    }
}

//The result of S(30) is given.  In my investigations is notices that for some values of n; there are multiple
// smaller periodic solutions.  So for S(30) this code does not produce always the correct answer.  The
//(unproved) assumption of this code is that when the found period is sufficiently large; the is only one cyclic solution
//for n.
fn main() {
    let min = 3;
    let mid = 30;
    let max = 100;
    let (first_half, second_half) = (min..=mid, mid + 1..=max);

    //by using into_iter() or inti_par_iter() we switch between single threaded and multithreaded, thank to the rayon crate.
    let mut result: HashSet<usize> = first_half
        .into_iter()
        .map(|i| {
            let mycircle = Circle::new(i);
            let current = Instant::now();
            let p = multi_stack(mycircle).unwrap();
            let duration = current.elapsed();
            println!("{:?} {:?} {:?}", i, p, duration);
            p
        })
        .collect();

    let intermediate_sum: usize = result.iter().sum();

    println!("Intermediate sum: {:?}", intermediate_sum);

    let temp: Vec<usize> = second_half
        .into_par_iter()
        .map(|i| {
            let mycircle = Circle::new(i);
            let current = Instant::now();
            let p = multi_stack(mycircle).unwrap();
            let duration = current.elapsed();
            println!("{:?} {:?} {:?}", i, p, duration);
            p
        })
        .collect();
    result.extend(temp);

    let sum: usize = result.iter().sum();

    println!("Total sum: {:?}", sum);
}
