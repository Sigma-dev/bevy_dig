use std::fmt::Debug;

pub fn print_slices(array_3d: &[impl Debug]) {
    let len = array_3d.len();
    println!("{}", len);
    let size = ((len as f32).cbrt().round() + 0.1) as usize;
    if size * size * size != len {
        panic!("Not 3d array");
    }
    for z in 0..size {
        let mut str = String::new();
        for y in 0..size {
            for x in 0..size {
                str.push_str(format!("{:?}, ", array_3d[x + y * size + z * size * size],).as_str());
            }
            str.push('\n');
        }
        println!("Slice {}:\n{}", z, str);
    }
}
