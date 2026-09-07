use ::higher_kinded_types_proc_macros::map_lifetime;

// type X = map_lifetime!('_ => 'a in &'_ str);