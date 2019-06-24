use super::*;
use std::fs::File;
use std::hash::Hash;
use std::io::BufReader;

pub trait FileGetter {
    fn get(&mut self, &str) -> std::io::Result<BufReader<File>>;
}

pub struct ImportMason<F: FileGetter> {
    get_file: F,
}

impl<F: FileGetter> ImportMason<F> {
    pub fn new(get_file: F) -> ImportMason<F> {
        ImportMason { get_file }
    }
}

impl<F: FileGetter> StoneMason for ImportMason<F> {
    fn handle_stones(
        &mut self,
        arch: &mut Architect,
        map: &mut HashMap<String, Vec<usize>>,
    ) -> HashSet<usize> {
        let res = HashSet::new();
        if !map.contains_key("import") {
            return res;
        }
        let mut import = map.get("import").unwrap().clone();
        while !import.is_empty() {
            let i = import.pop().unwrap();
            let url = match arch.stones[i].value.as_el().attr.get("url") {
                Some(u) => match String::from_attr(u, &arch.stones, i) {
                    Ok(o) => o,
                    Err(_) => {
                        arch.errors.push(StoneError::from(SelectorError::Parse));
                        continue;
                    }
                },
                None => {
                    arch.errors.push(StoneError::MissingAttr("url".into()));
                    continue;
                }
            };
            let buffer = match self.get_file.get(&url[..]) {
                Ok(b) => b,
                Err(e) => {
                    arch.errors
                        .push(StoneError::InvalidAttr(format!("url: {}", e)));
                    continue;
                }
            };
            merge_hashmaps(map, arch.decode_buffer(buffer, i, true), |a, mut b| {
                a.append(&mut b)
            });
            if let Some(imp) = map.get("import") {
                import.append(&mut imp.clone())
            }
        }
        HashSet::new()
    }
}

pub fn merge_hashmaps<K: Eq + Hash, V: Eq + Hash>(
    a: &mut HashMap<K, V>,
    b: HashMap<K, V>,
    merge_v: fn(&mut V, V),
) -> &mut HashMap<K, V> {
    for (k, bv) in b {
        if a.contains_key(&k) {
            merge_v(a.get_mut(&k).unwrap(), bv)
        } else {
            a.insert(k, bv);
        }
    }
    a
}
