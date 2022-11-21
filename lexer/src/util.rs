pub fn ignore_num_ref(x: Option<&(usize, char)>) -> Option<char> {
    x.map(|x| x.1)
}

pub fn _ignore_num(x: Option<(usize, char)>) -> Option<char> {
    x.map(|x| x.1)
}
