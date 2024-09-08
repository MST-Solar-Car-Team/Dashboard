// Converts number to float
pub fn f<T>(num: T) -> f64  
where 
f64: From<T>
{ 
    f64::from(num)
}