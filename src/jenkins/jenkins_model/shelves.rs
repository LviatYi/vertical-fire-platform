use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Default)]
pub struct Shelves(pub Vec<u32>);

impl FromStr for Shelves {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut v: u32 = 0;
        let mut result = Vec::new();
        for char in s.to_string().chars() {
            if !char.is_ascii_digit() {
                if v != 0 {
                    result.push(v);
                    v = 0;
                }
            } else {
                v *= 10;
                v += char.to_digit(10).unwrap();
            }
        }

        if v != 0 {
            result.push(v);
        }

        Ok(Self(result))
    }
}

impl Display for Shelves {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl FromIterator<u32> for Shelves {
    fn from_iter<T: IntoIterator<Item = u32>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let shelves = Shelves::from_str("1,2,3").unwrap();
        assert_eq!(shelves.0, vec![1, 2, 3]);

        let shelves = Shelves::from_str("123,234,345").unwrap();
        assert_eq!(shelves.0, vec![123, 234, 345]);

        let shelves = Shelves::from_str("123 234 |345").unwrap();
        assert_eq!(shelves.0, vec![123, 234, 345]);
    }
}
