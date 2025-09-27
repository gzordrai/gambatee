pub enum Rarety {
    Common(f32),
    Rare(f32),
    Epic(f32),
    Legendary(f32),
}

impl Rarety {
    pub fn drop_rate(&self) -> f32 {
        match self {
            Rarety::Common(rate) => *rate,
            Rarety::Rare(rate) => *rate,
            Rarety::Epic(rate) => *rate,
            Rarety::Legendary(rate) => *rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Rarety;

    #[test]
    fn test_common_drop_date() {
        let common = Rarety::Common(50.0);

        assert_eq!(common.drop_rate(), 50.0)
    }

    #[test]
    fn test_rare_drop_date() {
        let rare = Rarety::Rare(50.0);

        assert_eq!(rare.drop_rate(), 50.0)
    }

    #[test]
    fn test_epic_drop_date() {
        let epic = Rarety::Epic(50.0);

        assert_eq!(epic.drop_rate(), 50.0)
    }

    #[test]
    fn test_legendary_drop_date() {
        let legendary = Rarety::Legendary(50.0);

        assert_eq!(legendary.drop_rate(), 50.0)
    }
}
