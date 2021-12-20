pub use std::f32::consts::PI;

// I feel like these are way too "bunchy". 
// Like, I'll get stuck at far ends for d. 
// What I want is something linear, but that's scaled by an amount relative to my canvas. 
// I think we can actually do it that way. But for now, use these functions. Although I really think they do "lose" data. 
// I mean, it just doesn't make sense to me to bunch them like that. I can see how they're a "convenient" approximation to use, 
// which do yield you results, but I feel like they error on the side of unnecessarily loses information and creating artifacts. 
// Hmm yes, this is probably why they say to use the ReLU function. 
// Can definitely think more about these. But for now, I would say these function below make "more" sense on the squashing of the neurons. 
// But we want something closer to linear for the outputs. 

// sigmoid activation function
pub fn sigmoid(x:f32) ->f32 { // Range (0,1)
    1.0 / (1.0 + f32::exp(-x))
}

// tanh activation function
pub fn tanh(x:f32) ->f32 { // yields Range(-1,1)
    let e1 = f32::exp(x);
    let e2 = f32::exp(-x);
    (e1 - e2) / (e1 + e2)
    // (exp(x) - exp(-x)) / (exp(x) + exp(-x))
}
 
// rectified linear function
pub fn rectified(x:f32) ->f32 {
    f32::max(0.0, x)
}
